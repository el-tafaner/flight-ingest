# ADR 0001: Rust como lenguaje de flight-ingest

- **Estado:** aceptada
- **Fecha:** 2026-11-07

## Contexto
Ingesta y validación de datos de vuelos en tiempo real.

## Decisión
Rust.

## Motivo real
Aprender el lenguaje: está en auge y el autor no lo había usado.
No es una decisión de rendimiento, el volumen de datos no lo exige.

## Ventajas técnicas secundarias
Tipos fuertes para modelar el dominio, errores explícitos con `Result`
y concurrencia segura.

## Alternativas descartadas
- **Python:** más rápido de escribir y mejor ecosistema de datos.
- **Go:** buen candidato para servicios de red.

Ninguna es peor técnicamente; simplemente no enseñaban lo que se
quería aprender.

## Consecuencias
Curva de aprendizaje y más código para lo mismo, a cambio de entender
un modelo de memoria distinto.