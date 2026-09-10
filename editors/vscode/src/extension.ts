import {
  window,
  workspace,
  ExtensionContext,
  ShellExecution,
  Task,
  TaskPanelKind,
  TaskRevealKind,
  TaskScope,
  tasks,
  commands,
} from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';
import * as fs from 'fs';
import * as path from 'path';

let client: LanguageClient | undefined;

interface ZetteltexTarget {
  kind: 'note' | 'project';
  name: string;
}

/** Minimal workspace structure that identifies a ZettelTeX workspace root. */
const REQUIRED_STRUCTURE = ['notes/slipbox', 'projects', 'template'];

/**
 * Walk up from `start` (or the first workspace folder) until a directory that
 * looks like a ZettelTeX workspace root (contains notes/slipbox, projects and
 * template) is found. This lets the extension work even when only a project
 * subfolder is open as the workspace.
 */
function findWorkspaceRoot(start?: string): string | undefined {
  const candidates: string[] = [];
  if (start) {
    candidates.push(start);
  }
  const folder = workspace.workspaceFolders?.[0];
  if (folder) {
    candidates.push(folder.uri.fsPath);
  }
  for (const startDir of candidates) {
    let dir = startDir;
    for (;;) {
      if (
        REQUIRED_STRUCTURE.every((entry) =>
          fs.existsSync(path.join(dir, entry)),
        )
      ) {
        return dir;
      }
      const parent = path.dirname(dir);
      if (parent === dir) {
        break;
      }
      dir = parent;
    }
  }
  return undefined;
}

function stripTexSuffix(name: string): string {
  return name.toLowerCase().endsWith('.tex') ? name.slice(0, -4) : name;
}

/**
 * Map a file under the ZettelTeX workspace root to the note or project it
 * belongs to: notes live under `notes/slipbox`, projects under
 * `projects/<name>/` (any file in the folder targets that project).
 */
function targetForFile(
  root: string,
  filePath: string,
): ZetteltexTarget | undefined {
  const rel = path.relative(root, filePath);
  if (rel.startsWith('..') || path.isAbsolute(rel)) {
    return undefined;
  }
  const parts = rel.split(path.sep);
  if (parts[0] === 'notes') {
    const base = parts[1] === 'slipbox' ? parts.slice(2) : parts.slice(1);
    const name = stripTexSuffix(base.join('/'));
    return name ? { kind: 'note', name } : undefined;
  }
  if (parts[0] === 'projects' && parts.length >= 2) {
    return { kind: 'project', name: parts[1] };
  }
  return undefined;
}

/**
 * If the open workspace folder is itself a project folder (e.g.
 * `projects/<name>` was opened directly), that project is the target.
 */
function projectOfOpenFolder(root: string): ZetteltexTarget | undefined {
  const folder = workspace.workspaceFolders?.[0];
  if (!folder) {
    return undefined;
  }
  const projectsDir = path.join(root, 'projects');
  const rel = path.relative(projectsDir, folder.uri.fsPath);
  if (rel.startsWith('..') || path.isAbsolute(rel)) {
    return undefined;
  }
  const name = rel.split(path.sep)[0];
  if (!name) {
    return undefined;
  }
  const main = path.join(projectsDir, name, `${name}.tex`);
  if (fs.existsSync(main)) {
    return { kind: 'project', name };
  }
  return undefined;
}

/** Resolve the note/project the user currently has in focus or open. */
function currentTarget(root: string): ZetteltexTarget | undefined {
  const editor = window.activeTextEditor;
  if (editor) {
    const fromFile = targetForFile(root, editor.document.uri.fsPath);
    if (fromFile) {
      return fromFile;
    }
  }
  return projectOfOpenFolder(root);
}

/**
 * Directory that contains the TeX binaries (`pdflatex`, `biber`, `synctex`).
 * The `zetteltex.tex.binDir` setting wins; otherwise well-known TeX Live
 * locations are probed. GUI-launched VS Code often lacks the TeX Live bin dir
 * on its PATH, so it is prepended to the environment of every zetteltex task.
 */
function findTexBinDir(): string | undefined {
  const configured = workspace
    .getConfiguration('zetteltex')
    .get<string>('tex.binDir', '');
  if (configured && fs.existsSync(path.join(configured, 'pdflatex'))) {
    return configured;
  }
  for (const root of ['/usr/local/texlive', '/opt/texlive']) {
    if (!fs.existsSync(root)) {
      continue;
    }
    // Sort by year descending so the newest TeX Live tree wins.
    const years = fs
      .readdirSync(root)
      .filter((name) => /^\d{4}$/.test(name))
      .sort()
      .reverse();
    for (const name of years) {
      const bins = path.join(root, name, 'bin');
      if (!fs.existsSync(bins)) {
        continue;
      }
      for (const arch of fs.readdirSync(bins)) {
        const dir = path.join(bins, arch);
        if (fs.existsSync(path.join(dir, 'pdflatex'))) {
          return dir;
        }
      }
    }
  }
  return undefined;
}

/** Run `zetteltex <args>` from the workspace root using a VS Code task. */
async function runCli(root: string, args: string[]): Promise<void> {
  const configured = workspace
    .getConfiguration('zetteltex')
    .get<string>('cli.path', 'zetteltex');
  const binary = configured || 'zetteltex';

  const quoted = args.map((arg) => `"${arg.replace(/"/g, '\\"')}"`);
  const label = `zetteltex: ${args.join(' ')}`;
  const extraEnv = workspace
    .getConfiguration('zetteltex')
    .get<Record<string, string>>('cli.env', {});
  const texBinDir = findTexBinDir();
  const basePath = process.env.PATH ?? '';
  const env: Record<string, string> = {
    ...(process.env as Record<string, string>),
    ...extraEnv,
  };
  if (texBinDir && basePath) {
    env.PATH = `${texBinDir}${path.delimiter}${basePath}`;
  } else if (texBinDir) {
    env.PATH = texBinDir;
  }
  const execution = new ShellExecution(
    `"${binary}" --workspace-root "${root}" ${quoted.join(' ')}`.trim(),
    { cwd: root, env },
  );
  // The task source/name doubles as the terminal name, so repeated renders
  // reuse the same terminal and `watch` gets a dedicated, killable one.
  const task = new Task(
    { type: 'zetteltex-cli', name: args[0] },
    TaskScope.Workspace,
    label,
    'zetteltex',
    execution,
    [],
  );
  task.presentationOptions = {
    reveal: TaskRevealKind.Always,
    panel: TaskPanelKind.Dedicated,
    focus: true,
    clear: false,
  };
  await tasks.executeTask(task);
}

async function renderCurrent(): Promise<void> {
  const root = findWorkspaceRoot();
  if (!root) {
    void window.showErrorMessage(
      'No se encontró una raíz de ZettelTeX (hace falta notes/slipbox, projects, template). ' +
        'Abre una carpeta dentro del workspace.',
    );
    return;
  }
  const target = currentTarget(root);
  if (!target) {
    void window.showWarningMessage(
      'Enfoca una nota o abre un proyecto de ZettelTeX para poder renderizarlo.',
    );
    return;
  }
  const args =
    target.kind === 'project'
      ? ['render', '--project', target.name]
      : ['render', target.name];
  await runCli(root, args);
}

export function activate(context: ExtensionContext): Thenable<void> {
  const outputChannel = window.createOutputChannel('ZettelTeX Language Server');

  context.subscriptions.push(
    commands.registerCommand('zetteltex.render', renderCurrent),
  );

  const configured = workspace
    .getConfiguration('zetteltex')
    .get<string>('lsp.path', 'zetteltex');
  const command = configured || 'zetteltex';

  const root = findWorkspaceRoot() ?? process.cwd();
  const args = ['--workspace-root', root, 'lsp'];

  const serverOptions: ServerOptions = {
    command,
    args,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'latex' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.tex'),
    },
    outputChannel,
  };

  client = new LanguageClient(
    'zetteltex',
    'ZettelTeX Language Server',
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(outputChannel);
  return client.start().catch((err: unknown) => {
    const msg = err instanceof Error ? err.message : String(err);
    void window.showErrorMessage(`ZettelTeX LSP failed to start: ${msg}`);
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}