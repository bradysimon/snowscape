import * as vscode from "vscode";
import { PreviewCodeLensProvider } from "./previewCodeLensProvider";

/**
 * Activate the extension
 */
export function activate(context: vscode.ExtensionContext) {
  console.log("Snowscape extension is now active");

  // Register the Code Lens provider for Rust files
  const codeLensProvider = new PreviewCodeLensProvider();
  const codeLensDisposable = vscode.languages.registerCodeLensProvider(
    { language: "rust", scheme: "file" },
    codeLensProvider
  );

  // Register the run preview command
  const runPreviewDisposable = vscode.commands.registerCommand(
    "snowscape.runPreview",
    async (uri: vscode.Uri, functionName: string, previewType: string) => {
      await runPreview(uri, functionName, previewType);
    }
  );

  // Register a command to refresh code lenses (useful for debugging)
  const refreshDisposable = vscode.commands.registerCommand(
    "snowscape.refreshCodeLenses",
    () => {
      codeLensProvider.refresh();
    }
  );

  context.subscriptions.push(
    codeLensDisposable,
    runPreviewDisposable,
    refreshDisposable
  );

  // Show a message when the extension is activated
  vscode.window.showInformationMessage("Snowscape extension loaded");
}

/**
 * Run a preview in a terminal
 */
async function runPreview(
  uri: vscode.Uri,
  functionName: string,
  previewType: string
): Promise<void> {
  // Find the workspace folder containing the file
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder found for this file");
    return;
  }

  // Get the configured preview command
  const config = vscode.workspace.getConfiguration("snowscape");
  const baseCommand = config.get<string>(
    "previewCommand",
    "cargo run --bin preview"
  );

  // Auto-detect the correct cargo package based on file location
  const command = await buildCargoCommand(baseCommand, uri, workspaceFolder);

  // Create or reuse a terminal for running the preview
  const terminalName = `Snowscape: ${functionName}`;

  // Check if a terminal with this name already exists
  let terminal = vscode.window.terminals.find((t) => t.name === terminalName);

  if (!terminal) {
    terminal = vscode.window.createTerminal({
      name: terminalName,
      cwd: workspaceFolder.uri.fsPath,
      // Use a clean environment
      env: {},
    });
  }

  // Show the terminal and run the command with preview selection
  terminal.show(true); // true = preserve focus on editor

  // Get the preview selection method from configuration
  const selectionMethod = config.get<string>(
    "previewSelectionMethod",
    "environment"
  );

  let finalCommand: string;
  if (selectionMethod === "argument") {
    // Use command-line argument: cargo run --bin preview --preview=functionName
    finalCommand = `${command} --preview=${functionName}`;
  } else {
    // Use environment variable: SNOWSCAPE_PREVIEW="functionName" cargo run --bin preview
    finalCommand = `SNOWSCAPE_PREVIEW="${functionName}" ${command}`;
  }

  terminal.sendText(finalCommand);

  // Show a notification
  vscode.window.showInformationMessage(
    `Running ${previewType} preview: ${functionName}`
  );
}

/**
 * Build the appropriate cargo command based on the file location and workspace structure
 */
async function buildCargoCommand(
  baseCommand: string,
  uri: vscode.Uri,
  workspaceFolder: vscode.WorkspaceFolder
): Promise<string> {
  const fs = vscode.workspace.fs;

  // Get the file path relative to workspace root
  const relativePath = vscode.workspace.asRelativePath(uri, false);

  // Check if we're in a workspace with multiple packages
  try {
    // Look for Cargo.toml files to detect package structure
    const workspaceCargoToml = vscode.Uri.joinPath(
      workspaceFolder.uri,
      "Cargo.toml"
    );

    try {
      const workspaceTomlContent = await fs.readFile(workspaceCargoToml);
      const workspaceTomlText =
        Buffer.from(workspaceTomlContent).toString("utf8");

      // Check if this is a workspace (contains [workspace] section)
      if (workspaceTomlText.includes("[workspace]")) {
        // Try to find the specific package this file belongs to
        const packageName = await findPackageForFile(
          fs,
          workspaceFolder,
          relativePath
        );

        if (packageName) {
          // Replace the base command with package-specific version
          // Handle various command formats
          if (baseCommand.includes("cargo run --bin preview")) {
            return baseCommand.replace(
              "cargo run --bin preview",
              `cargo run -p ${packageName} --bin preview`
            );
          } else if (baseCommand.includes("cargo run")) {
            // For custom commands, try to add -p flag after "cargo run"
            return baseCommand.replace(
              "cargo run",
              `cargo run -p ${packageName}`
            );
          }
        }
      }
    } catch (error) {
      // Workspace Cargo.toml doesn't exist or can't be read, continue with single package logic
    }

    // For single-package projects or if we can't detect the package, use the base command as-is
    return baseCommand;
  } catch (error) {
    console.warn("Error detecting cargo workspace structure:", error);
    return baseCommand;
  }
}

/**
 * Find which cargo package a file belongs to by looking for the nearest Cargo.toml
 */
async function findPackageForFile(
  fs: vscode.FileSystem,
  workspaceFolder: vscode.WorkspaceFolder,
  relativePath: string
): Promise<string | null> {
  const pathParts = relativePath.split("/");

  // Work backwards from the file to find a Cargo.toml
  for (let i = pathParts.length - 1; i > 0; i--) {
    const packagePath = pathParts.slice(0, i).join("/");
    const cargoTomlPath = vscode.Uri.joinPath(
      workspaceFolder.uri,
      packagePath,
      "Cargo.toml"
    );

    try {
      const cargoTomlContent = await fs.readFile(cargoTomlPath);
      const cargoTomlText = Buffer.from(cargoTomlContent).toString("utf8");

      // Extract package name from [package] section
      const packageMatch = cargoTomlText.match(
        /\[package\]\s*\n(?:[^\[]*\n)*?name\s*=\s*"([^"]+)"/
      );
      if (packageMatch) {
        return packageMatch[1];
      }
    } catch (error) {
      // Cargo.toml doesn't exist at this level, continue searching
      continue;
    }
  }

  return null;
}

/**
 * Deactivate the extension
 */
export function deactivate() {
  console.log("Snowscape extension deactivated");
}
