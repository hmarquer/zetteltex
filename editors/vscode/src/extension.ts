import {
  window,
  workspace,
  ExtensionContext,
  StatusBarAlignment,
  commands,
  OutputChannel,
  StatusBarItem,
  TextDocument,
} from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';
import * as cp from 'child_process';
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

/** Environment for CLI subprocesses: extra env vars plus the TeX bin dir. */
function cliEnv(): Record<string, string> {
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
  return env;
}

function cliPath(): string {
  const configured = workspace
    .getConfiguration('zetteltex')
    .get<string>('cli.path', 'zetteltex');
  return configured || 'zetteltex';
}

let cliOutput: OutputChannel | undefined;

function getCliOutput(): OutputChannel {
  if (!cliOutput) {
    cliOutput = window.createOutputChannel('ZettelTeX');
  }
  return cliOutput;
}

/** Running `watch` processes, keyed by the CLI args used to start them. */
const runningWatches = new Map<string, cp.ChildProcess>();

const watchStatus = window.createStatusBarItem(
  StatusBarAlignment.Right,
  100,
);

function updateWatchStatusBar(): void {
  if (runningWatches.size > 0) {
    watchStatus.text = `$(sync~spin) ZettelTeX: ${runningWatches.size} watch`;
    watchStatus.tooltip =
      [...runningWatches.keys()].join('\n') + '\nClic para detener';
    watchStatus.command = 'zetteltex.watchStop';
    watchStatus.show();
  } else {
    watchStatus.hide();
  }
}

/**
 * Start a long-running `zetteltex watch` as a background process. The output
 * goes to the ZettelTeX channel; a status-bar item (click to stop) shows while
 * it is alive, and `zetteltex.watchStop` stops every running watch.
 */
async function startWatch(root: string, args: string[]): Promise<void> {
  const binary = cliPath();
  const key = args.join(' ');
  if (runningWatches.has(key)) {
    void window.showInformationMessage(
      `Ya hay un watch en ejecución para '${key}'.`,
    );
    return;
  }
  const label = `zetteltex: ${key}`;
  const fullArgs = ['--workspace-root', root, ...args];
  const output = getCliOutput();
  output.appendLine(`> ${binary} ${fullArgs.join(' ')}`);

  const child = cp.spawn(binary, fullArgs, {
    cwd: root,
    env: cliEnv(),
  });
  runningWatches.set(key, child);
  updateWatchStatusBar();

  child.stdout?.on('data', (chunk) => output.append(chunk.toString()));
  child.stderr?.on('data', (chunk) => output.append(chunk.toString()));
  child.on('error', (err) => {
    output.appendLine(`ERROR: ${err.message}`);
    runningWatches.delete(key);
    updateWatchStatusBar();
    void window.showErrorMessage(`${label}: ${err.message}`);
  });
  child.on('close', (code) => {
    runningWatches.delete(key);
    updateWatchStatusBar();
    if (code !== undefined && code !== 0) {
      output.show(true);
      void window.showErrorMessage(`${label} terminó con código ${code}`);
    }
  });

  void window.showInformationMessage(`ZettelTeX watch iniciado: '${key}'.`);
}

async function stopWatch(): Promise<void> {
  const keys = [...runningWatches.keys()];
  if (keys.length === 0) {
    void window.showInformationMessage('No hay ningún watch en ejecución.');
    return;
  }
  const output = getCliOutput();
  output.show(true);
  output.appendLine('Deteniendo watch(es)...');
  for (const key of keys) {
    const child = runningWatches.get(key);
    if (child && !child.killed) {
      child.kill();
    }
  }
  updateWatchStatusBar();
}

async function runCli(root: string, args: string[]): Promise<void> {
  const binary = cliPath();
  const summary = args.join(' ');
  const label = `zetteltex: ${summary}`;
  const fullArgs = ['--workspace-root', root, ...args];
  const output = getCliOutput();
  if (runningWatches.size === 0) {
    output.clear();
  }
  output.appendLine(`> ${binary} ${fullArgs.join(' ')}`);

  renderStatus.text = `$(sync~spin) ZettelTeX: ${summary}`;
  renderStatus.tooltip = label;
  renderStatus.show();

  await new Promise<void>((resolve) => {
    const child = cp.spawn(binary, fullArgs, {
      cwd: root,
      env: cliEnv(),
    });
    child.stdout?.on('data', (chunk) => output.append(chunk.toString()));
    child.stderr?.on('data', (chunk) => output.append(chunk.toString()));

    const reportFinish = (code: number | null): void => {
      output.appendLine(
        code === 0
          ? 'ZettelTeX: listo.'
          : `ZettelTeX: terminó con código ${code ?? '?'}.`,
      );
      const finished =
        code === 0
          ? `$(check) ZettelTeX: ${summary} listo`
          : `$(error) ZettelTeX: ${summary} falló (${code ?? '?'})`;
      renderStatus.text = finished;
      renderStatus.tooltip = label;
      renderStatus.show();
      setTimeout(() => {
        if (renderStatus.text === finished) {
          renderStatus.hide();
        }
      }, 5000);
      resolve();
    };

    child.on('error', (err) => {
      const failed = `$(error) ZettelTeX: ${err.message}`;
      output.appendLine(`ERROR: ${err.message}`);
      renderStatus.text = failed;
      renderStatus.tooltip = label;
      renderStatus.show();
      setTimeout(() => {
        if (renderStatus.text === failed) {
          renderStatus.hide();
        }
      }, 5000);
      resolve();
    });
    child.on('close', reportFinish);
  });
}

/** Status bar item reporting one-shot CLI runs (render, ...). */
const renderStatus = window.createStatusBarItem(
  StatusBarAlignment.Right,
  99,
);

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

async function watchCurrent(): Promise<void> {
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
      'Enfoca una nota o abre un proyecto de ZettelTeX para poder vigilarlo.',
    );
    return;
  }
  const args =
    target.kind === 'project'
      ? ['watch', '--project', target.name]
      : ['watch', target.name];
  await startWatch(root, args);
}

async function watchWorkspace(): Promise<void> {
  const root = findWorkspaceRoot();
  if (!root) {
    void window.showErrorMessage(
      'No se encontró una raíz de ZettelTeX (hace falta notes/slipbox, projects, template). ' +
        'Abre una carpeta dentro del workspace.',
    );
    return;
  }
  await startWatch(root, ['watch']);
}

const RENDER_ON_SAVE_DEBOUNCE_MS = 300;
const renderOnSaveTimers = new Map<string, NodeJS.Timeout>();

/**
 * With `zetteltex.renderOnSave` enabled, rendering is triggered when a `.tex`
 * file of a note or of a project folder is saved. Saves of the same target are
 * debounced so a burst of saves (e.g. LaTeX Workshop touching multiple files)
 * does not spawn duplicate renders.
 */
function onDidSaveTextDocument(document: TextDocument): void {
  if (
    !workspace
      .getConfiguration('zetteltex')
      .get<boolean>('renderOnSave', false)
  ) {
    return;
  }
  if (!document.uri.fsPath.toLowerCase().endsWith('.tex')) {
    return;
  }
  const root = findWorkspaceRoot(document.uri.fsPath);
  if (!root) {
    return;
  }
  const target = targetForFile(root, document.uri.fsPath);
  if (!target) {
    return;
  }
  const key = `${target.kind}:${target.name}`;
  const previous = renderOnSaveTimers.get(key);
  if (previous) {
    clearTimeout(previous);
  }
  const timer = setTimeout(() => {
    renderOnSaveTimers.delete(key);
    const args =
      target.kind === 'project'
        ? ['render', '--project', target.name]
        : ['render', target.name];
    void runCli(root, args);
  }, RENDER_ON_SAVE_DEBOUNCE_MS);
  renderOnSaveTimers.set(key, timer);
}

export function activate(context: ExtensionContext): Thenable<void> {
  const outputChannel = window.createOutputChannel('ZettelTeX Language Server');

  context.subscriptions.push(
    commands.registerCommand('zetteltex.render', renderCurrent),
    commands.registerCommand('zetteltex.watch', watchCurrent),
    commands.registerCommand('zetteltex.watchWorkspace', watchWorkspace),
    commands.registerCommand('zetteltex.watchStop', stopWatch),
    watchStatus,
    renderStatus,
    workspace.onDidSaveTextDocument(onDidSaveTextDocument),
    {
      dispose: () => {
        for (const timer of renderOnSaveTimers.values()) {
          clearTimeout(timer);
        }
        renderOnSaveTimers.clear();
      },
    },
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