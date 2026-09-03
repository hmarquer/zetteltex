import {
  window,
  workspace,
  ExtensionContext,
} from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext): Thenable<void> {
  const outputChannel = window.createOutputChannel('ZettelTeX Language Server');

  const configured = workspace
    .getConfiguration('zetteltex')
    .get<string>('lsp.path', 'zetteltex');
  const command = configured || 'zetteltex';

  const folders = workspace.workspaceFolders;
  const root =
    folders && folders.length > 0 ? folders[0].uri.fsPath : process.cwd();
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
