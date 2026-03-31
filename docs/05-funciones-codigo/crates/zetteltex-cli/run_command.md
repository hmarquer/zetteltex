# run_command

## Firma
`fn run_command(command: Commands, paths: &WorkspacePaths) -> Result<ExitCode>`

## Responsabilidad
Despachar cada subcomando de CLI a su funcion operativa y normalizar codigo de salida exitoso.

## Flujo interno resumido
1. hace `match` sobre `Commands`.
2. llama funcion de dominio correspondiente.
3. retorna `ExitCode::SUCCESS` en ruta exitosa.

## Precondiciones
- `paths` ya validado por `WorkspacePaths::discover`.

## Postcondiciones
- comando ejecutado o error propagado al caller (`main`).

## Llamadores principales
- `main`

## Relacionado
- [Catalogo por comando](../../../03-comandos/README.md)
- [Modelo de workspace](../../../02-guia-tecnica/modelo-workspace.md)

## Ubicacion
- `crates/zetteltex-cli/src/main.rs`
