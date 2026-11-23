# Registro de Ventas en Solana

## Ruta elegida
Guiada: proyecto Registro / Cliente / Venta de las sesiones del curso, con una mejora propia.

## Propósito
Programa en Solana (Anchor) para que un negocio lleve su registro de clientes y ventas en la blockchain. Cada dueño tiene un registro propio (PDA) con sus clientes y el historial de compras de cada uno. Incluye un frontend en React con wallet para usarlo desde el navegador.

## Despliegue en Devnet
- **Program ID:** `788k8AagUMgf4zhJ519UxsF6QduZmfdFr4okVxURbWhg`
- [Ver en Solana Explorer](https://explorer.solana.com/address/788k8AagUMgf4zhJ519UxsF6QduZmfdFr4okVxURbWhg?cluster=devnet)

## Estructura
- `src/lib.rs`: programa de Solana
- `idl.json`: IDL del programa
- `client/`: cliente del programa
- `tests/`: pruebas
- `frontend/`: interfaz web (React + Vite + TypeScript)

## Reglas del programa
- **Autoridad:** solo el owner del registro puede registrar ventas, devoluciones, cambios o bajas (`NoOwner`).
- **Pertenencia:** el registro es un PDA con seeds `"registro"` + clave pública del owner.
- **Cierre de cuentas:** el registro solo se cierra si ya no tiene clientes (`RegistroConClientes`).

## Cómo ejecutarlo
Requisitos: Node.js y una wallet de Solana (Solflare o Phantom) en Devnet con SOL.

    cd frontend
    npm install
    npm run dev

Abre http://localhost:5173, conecta la wallet y usa los tres bloques: crear registro, registrar venta y leer datos.

## Mi aporte propio
Agregué una validación en el programa Rust: `registrar_venta` rechaza ventas con monto 0 (`require!(monto > 0, Errores::MontoInvalido)`, `src/lib.rs`, línea 64). Evita guardar ventas sin valor y devuelve un error claro (código 6004).

También construí el frontend y el cliente en TypeScript que conectan la wallet con el programa.

## Cómo probarlo
1. Conecta la wallet en Devnet.
2. **Crear registro** con un nombre de negocio.
3. **Registrar venta** con ID cliente `0` (crea un cliente nuevo), SKU, descripción y monto mayor a 0.
4. **Leer datos** muestra el registro, los clientes y su historial.
5. **Prueba de rechazo:** registrar una venta con monto `0` falla con `MontoInvalido` (6004).

## Evidencias (Devnet)
- Crear registro: (https://explorer.solana.com/tx/2pL2MbgVzasgEoXJeX17vVN6n2fNhrXsR4vKaG3LfnbQtV6q2afiiUkXJvU5B75M8tzuEBzyZz6ZAYED8GvbWDnU?cluster=devnet)
- Registrar venta: (https://explorer.solana.com/tx/4LHBfoAhbHuGssBPZMenhDzinnwg3iZg61AEy4w7kVvUy7ULiUvgCL6RhcgcrLxqBKfGprQDwfRbhkcwT6sg63ca?cluster=devnet)
- Rechazo con monto 0 (MontoInvalido): - Rechazo con monto 0 (MontoInvalido): captura incluida en la entrega