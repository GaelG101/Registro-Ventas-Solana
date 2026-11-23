// No imports needed: web3, anchor, pg and more are globally available

describe("Registro de Ventas", function () {
  this.timeout(600000);

  const owner = pg.wallet.publicKey;
  const [pdaRegistro] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("registro"), owner.toBuffer()],
    pg.PROGRAM_ID
  );
  const cuentas = { owner: owner, registro: pdaRegistro };

  // Envia la transaccion y espera a que quede finalizada en Devnet
  async function enviar(promesa) {
    const tx = await promesa;
    await pg.connection.confirmTransaction(tx, "finalized");
    return tx;
  }

  // Lee el registro reintentando hasta que se cumpla la condicion (Devnet puede ir atrasado)
  async function leerRegistro(condicion) {
    let ultimo = null;
    for (let i = 0; i < 30; i++) {
      try {
        ultimo = await pg.program.account.registro.fetch(pdaRegistro, "finalized");
        if (condicion(ultimo)) return ultimo;
      } catch (e) {
        ultimo = null; // todavia no existe
      }
    }
    return ultimo;
  }

  // Verifica que una transaccion FALLE con el error esperado
  async function esperarError(promesa, nombreError) {
    try {
      await promesa;
    } catch (e) {
      const detalle =
        (e?.error?.errorCode?.code ?? "") +
        " " +
        (e?.message ?? "") +
        " " +
        (e?.logs ?? []).join(" ");
      assert(
        detalle.includes(nombreError),
        `Se esperaba ${nombreError}, pero llego: ${detalle}`
      );
      return;
    }
    assert.fail(`Se esperaba el error ${nombreError}, pero la transaccion tuvo exito`);
  }

  // Limpieza: si quedo un registro de una corrida anterior, se vacia y se cierra
  before(async () => {
    const info = await pg.connection.getAccountInfo(pdaRegistro, "finalized");
    if (info === null) return;
    const reg = await pg.program.account.registro.fetch(pdaRegistro, "finalized");
    for (const c of reg.clientes) {
      await enviar(pg.program.methods.darBajaCliente(c.idCliente).accounts(cuentas).rpc());
    }
    await enviar(pg.program.methods.cerrarRegistro().accounts(cuentas).rpc());
  });

  it("crea el registro", async () => {
    await enviar(
      pg.program.methods.crearRegistro("La tiendita de Gael").accounts(cuentas).rpc()
    );
    const reg = await leerRegistro((r) => true);
    assert(reg !== null, "el registro no aparecio en Devnet");
    assert(reg.owner.equals(owner));
    assert.equal(reg.nombreNegocio, "La tiendita de Gael");
    assert.equal(reg.idCounter, 0);
    assert.equal(reg.clientes.length, 0);
  });

  it("registra una venta de cliente nuevo (id 0)", async () => {
    await enviar(
      pg.program.methods.registrarVenta(0, "Pan", "Pan bimbo", 48).accounts(cuentas).rpc()
    );
    const reg = await leerRegistro((r) => r.clientes.length === 1);
    assert.equal(reg.clientes.length, 1);
    assert.equal(reg.clientes[0].idCliente, 1);
    assert.equal(reg.clientes[0].compras, 1);
  });

  it("registra una venta de cliente existente", async () => {
    await enviar(
      pg.program.methods.registrarVenta(1, "Cafe", "Cafe del oxxo", 38).accounts(cuentas).rpc()
    );
    const reg = await leerRegistro((r) => r.clientes[0].compras === 2);
    assert.equal(reg.clientes[0].compras, 2);
  });

  it("cambia el monto de una venta", async () => {
    await enviar(pg.program.methods.cambiarMonto(1, "Pan", 50).accounts(cuentas).rpc());
    const reg = await leerRegistro((r) => r.clientes[0].historial[0].monto === 50);
    assert.equal(reg.clientes[0].historial[0].monto, 50);
  });

  // ---------- PRUEBAS DE RECHAZO ----------

  it("RECHAZO: no acepta una venta con monto 0", async () => {
    await esperarError(
      pg.program.methods.registrarVenta(1, "Leche", "Leche", 0).accounts(cuentas).rpc(),
      "MontoInvalido"
    );
  });

  it("RECHAZO: no acepta cambiar un monto a 0", async () => {
    await esperarError(
      pg.program.methods.cambiarMonto(1, "Pan", 0).accounts(cuentas).rpc(),
      "MontoInvalido"
    );
  });

  it("RECHAZO: otra wallet que no es el owner no puede modificar el registro", async () => {
    const intruso = new web3.Keypair();
    await esperarError(
      pg.program.methods
        .registrarVenta(1, "Sal", "Sal", 10)
        .accounts({ owner: intruso.publicKey, registro: pdaRegistro })
        .signers([intruso])
        .rpc(),
      "NoOwner"
    );
  });

  it("RECHAZO: no se puede cerrar el registro si todavia tiene clientes", async () => {
    await esperarError(
      pg.program.methods.cerrarRegistro().accounts(cuentas).rpc(),
      "RegistroConClientes"
    );
    // La cuenta debe seguir existiendo
    const reg = await leerRegistro((r) => r.clientes.length === 1);
    assert.equal(reg.clientes.length, 1);
  });

  it("RECHAZO: no se puede dar de baja un cliente que no existe", async () => {
    await esperarError(
      pg.program.methods.darBajaCliente(99).accounts(cuentas).rpc(),
      "NoUser"
    );
  });

  // ---------- CIERRE ----------

  it("da de baja al cliente", async () => {
    await enviar(pg.program.methods.darBajaCliente(1).accounts(cuentas).rpc());
    const reg = await leerRegistro((r) => r.clientes.length === 0);
    assert.equal(reg.clientes.length, 0);
  });

  it("cierra el registro (ya sin clientes) y recupera la renta", async () => {
    await enviar(pg.program.methods.cerrarRegistro().accounts(cuentas).rpc());

    let info = await pg.connection.getAccountInfo(pdaRegistro, "finalized");
    for (let i = 0; i < 10 && info !== null; i++) {
      info = await pg.connection.getAccountInfo(pdaRegistro, "finalized");
    }
    assert.equal(info, null); // la cuenta ya no existe
  });
});
