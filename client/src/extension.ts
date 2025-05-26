import * as net from "net";
import * as path from "path";
import { workspace, ExtensionContext, window } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  StreamInfo
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // Connect to an already running language server over TCP
  const serverOptions = () => {
    return new Promise<StreamInfo>((resolve, reject) => {
      const socket = net.connect(6009, "127.0.0.1", () => {
        resolve({
          reader: socket,
          writer: socket,
        });
      });

      socket.on("error", (err) => {
        reject(err);
      });
    });
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "sysy" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.sy")
    },
    outputChannelName: "SySy Language Server",
    revealOutputChannelOn: 4,
  };

  client = new LanguageClient(
    "sysy-language-server",
    "SySy Language Server",
    serverOptions,
    clientOptions
  );

  client.start().then(() => {
    window.showInformationMessage("SySy Language Server connected for debugging.");
  }).catch((error) => {
    window.showErrorMessage(`Failed to connect to SySy Language Server: ${error}`);
  });

  context.subscriptions.push(client.outputChannel);
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
