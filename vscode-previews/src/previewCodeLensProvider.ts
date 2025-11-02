import * as vscode from "vscode";

/**
 * Code Lens provider that detects Snowscape preview functions and adds "Run Preview" buttons
 */
export class PreviewCodeLensProvider implements vscode.CodeLensProvider {
  private _onDidChangeCodeLenses: vscode.EventEmitter<void> =
    new vscode.EventEmitter<void>();
  public readonly onDidChangeCodeLenses: vscode.Event<void> =
    this._onDidChangeCodeLenses.event;

  constructor() {
    // Refresh code lenses when configuration changes
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("snowscape.enableCodeLens")) {
        this._onDidChangeCodeLenses.fire();
      }
    });
  }

  /**
   * Provide code lenses for Snowscape preview functions
   */
  public provideCodeLenses(
    document: vscode.TextDocument,
    token: vscode.CancellationToken
  ): vscode.CodeLens[] | Thenable<vscode.CodeLens[]> {
    // Only activate for Rust files
    if (document.languageId !== "rust") {
      return [];
    }

    // Check if code lenses are enabled
    const config = vscode.workspace.getConfiguration("snowscape");
    const enabled = config.get<boolean>("enableCodeLens", true);
    if (!enabled) {
      return [];
    }

    const codeLenses: vscode.CodeLens[] = [];
    const text = document.getText();
    const lines = text.split("\n");

    // Regex to match #[snowscape::stateless] or #[snowscape::stateful(...)]
    const statelessRegex = /#\[snowscape::stateless(?:\([^)]*\))?\]/;
    const statefulRegex = /#\[snowscape::stateful\([^)]*\)\]/;

    // Track functions we've already added code lenses for to avoid duplicates
    const processedFunctions = new Set<string>();

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();

      // Check if this line has a snowscape attribute
      const isStateless = statelessRegex.test(line);
      const isStateful = statefulRegex.test(line);

      if (isStateless || isStateful) {
        const previewType = isStateless ? "stateless" : "stateful";

        // Find the function definition on the next non-empty, non-attribute line
        let functionLine = i + 1;
        while (functionLine < lines.length) {
          const nextLine = lines[functionLine].trim();

          // Skip empty lines and other attributes
          if (
            !nextLine ||
            nextLine.startsWith("#[") ||
            nextLine.startsWith("//")
          ) {
            functionLine++;
            continue;
          }

          // Check if this is a function definition
          if (
            nextLine.startsWith("fn ") ||
            nextLine.startsWith("pub fn ") ||
            nextLine.startsWith("async fn ") ||
            nextLine.startsWith("pub async fn ")
          ) {
            const functionNameMatch = nextLine.match(/fn\s+(\w+)/);
            if (functionNameMatch) {
              const functionName = functionNameMatch[1];
              const functionKey = `${functionLine}:${functionName}`;

              // Only add one code lens per function, even if it has multiple attributes
              if (!processedFunctions.has(functionKey)) {
                processedFunctions.add(functionKey);

                // Create a range for the code lens (on the function line)
                const range = new vscode.Range(
                  functionLine,
                  0,
                  functionLine,
                  0
                );

                // Create the code lens with command
                const codeLens = new vscode.CodeLens(range, {
                  title: `▶ Run Preview`,
                  tooltip: `Run ${functionName} ${previewType} preview`,
                  command: "snowscape.runPreview",
                  arguments: [document.uri, functionName, previewType],
                });

                codeLenses.push(codeLens);
              }
            }
          }
          break;
        }
      }
    }

    return codeLenses;
  }

  /**
   * Resolve a code lens (optional, we do everything in provideCodeLenses)
   */
  public resolveCodeLens(
    codeLens: vscode.CodeLens,
    token: vscode.CancellationToken
  ): vscode.CodeLens {
    return codeLens;
  }

  /**
   * Manually refresh code lenses
   */
  public refresh(): void {
    this._onDidChangeCodeLenses.fire();
  }
}
