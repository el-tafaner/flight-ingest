# ADR 0002: CI con versión fija de Rust

- **Estado:** aceptada
- **Fecha:** 2026-11-07

## Contexto
En v0.3, `clippy` 1.99 rechazó código que un compilador más antiguo
aceptaba (`clippy::collapsible_if`). Si el CI usara siempre la última
versión estable, un aviso nuevo podría romperlo sin que cambiara el código.

## Decisión
El CI instala una versión concreta de Rust (1.99.1) y la subimos a mano.

## Motivo
Es un proyecto didáctico: quien clone un hito antiguo debería ver el CI
en verde. Además, subir la versión pasa a ser una decisión consciente,
con su commit y sus avisos corregidos.

## Alternativas descartadas
- **`stable` siempre:** más simple y sin mantenimiento, pero el CI puede
  romperse solo.

## Consecuencias
Hay que acordarse de actualizar la versión de vez en cuando. Si se sube,
conviene actualizar también `rust-version` en `Cargo.toml` si hace falta.
