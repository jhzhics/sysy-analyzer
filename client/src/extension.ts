import * as path from "path";
import { workspace, ExtensionContext, window } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // The server is implemented in Rust
  const serverPath = context.asAbsolutePath(
    path.join("server", "target", "release", "sysy-analyzer-server")
  );

  // Show message when activating
  window.showInformationMessage('SySy Language Server is starting...');

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  const serverOptions: ServerOptions = {
    run: { command: serverPath, args: [] },
    debug: {
      command: serverPath,
      args: ["--debug"]
    },
  };

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for specific file types (change this to your language)
    documentSelector: [{ scheme: "file", language: "sysy" }],
    synchronize: {
      // Notify the server about file changes
      fileEvents: workspace.createFileSystemWatcher("**/*.sy")
    },
    outputChannelName: "SySy Language Server",
    revealOutputChannelOn: 4  // Show output on error
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    "sysy-language-server",
    "SySy Language Server",
    serverOptions,
    clientOptions
  );

  // Start the client. This will also launch the server
  client.start().then(() => {
    window.showInformationMessage('SySy Language Server is now active!');
  }).catch((error) => {
    window.showErrorMessage(`SySy Language Server failed to start: ${error}`);
  });

  // Register to the output channel
  context.subscriptions.push(client.outputChannel);
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}