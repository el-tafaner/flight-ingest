# flight-ingest

Rust library to fetch and validate real-time flight data. Didactic project: docs in Spanish, code in English.

Biblioteca en Rust para obtener y validar datos de vuelos en tiempo real. Es un proyecto didáctico: cada hito añade una pieza, con tests y una nota que explica el porqué de cada decisión, incluidos los errores por el camino.

> **Estado:** en construcción. Por ahora existen el dominio y la validación; el cliente de la API de vuelos (OpenSky) llega en el siguiente hito.

## Qué hay hoy

- `Icao24`: el identificador de cada aeronave, imposible de construir con un valor inválido.
- `Position` y `FlightState`: el estado de un vuelo, donde lo que puede faltar es `Option`.
- `validate` y `partition`: detectan datos implausibles y los separan **con el motivo**, sin descartarlos en silencio.

```rust
use flight_ingest::domain::Icao24;

assert_eq!(Icao24::parse("4840D6").unwrap().as_str(), "4840d6");
assert!(Icao24::parse("zzzzzz").is_err());
```

## Recorrido de aprendizaje

Cada hito tiene su tag de Git y su nota. Para ver el proyecto tal como estaba en uno de ellos: `git checkout v0.1`.

| Hito | Qué se construye | Nota |
|---|---|---|
| `v0.1` | `Icao24` y el patrón *newtype* | [v0.1: el dominio](docs/es/fases/v0.1-dominio.md) |
| `v0.2` | `Position`, `FlightState` y `Option` | [v0.2: FlightState](docs/es/fases/v0.2-flightstate.md) |
| `v0.3` | Errores con `thiserror` y validación | [v0.3: validación](docs/es/fases/v0.3-validacion.md) |
| `v0.4` | `FlightSource` y parser de OpenSky | [v0.4: fuente y parser](docs/es/fases/v0.4-fuente-y-parser.md) |
| `v0.5` | Cliente HTTP con OAuth2 y límites de créditos | próximamente |
| `v0.6` | Polling con presupuesto de créditos | próximamente |

Las decisiones importantes están en [`docs/es/adr/`](docs/es/adr/), incluida la de por qué Rust ([ADR 0001](docs/es/adr/0001-rust-como-lenguaje.md)): para aprender el lenguaje.

## Cómo probarlo

Necesitas Rust 1.88 o superior.

```bash
git clone https://github.com/el-tafaner/flight-ingest.git
cd flight-ingest
cargo test
```

## Cómo se trabaja aquí

- Primero el test, luego el código.
- `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test` en verde antes de cada commit.
- Explicaciones en castellano, código en inglés.

## Licencia

MIT. Ver [LICENSE](LICENSE).
