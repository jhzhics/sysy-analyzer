# A Sysy Analyzer
![Overview](assets/sysy-analyzer.png)

# Dependency
## Tree-sitter
- Install tree-sitter CLI

# Project Structure
- `.vscode/`: VsCode configuration files for building and debugging
- `client/`: Contains the client code
- `server/`: Contains the server code
- `syntax/`: Contains TextMate syntax definitions for coloring at grammar level


# Features
- Syntax highlighting using TextMate grammar
- Completion
- Go to definition
- Definition Hover

# Usage
Run vscode launch task `Debug Client + Server`

# Publish
## Package
```bash
npm install -g vsce
vsce package
```
## Package
```bash
vsce login <publisher-name>
vsce publish <version>
```