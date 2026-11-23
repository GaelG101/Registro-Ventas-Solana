use anchor_lang::prelude::*;
// ID del Solana Program, este espacio se llena automaticamente al hacer el "build"
declare_id!("788k8AagUMgf4zhJ519UxsF6QduZmfdFr4okVxURbWhg");

#[program] // Macro que convierte codigo de Rust a Solana.
pub mod registro_ventas {
    use super::*; // Importa todos los structs y enums definidos fuera del modulo

    //////////////////////////// Instruccion: Crear Registro /////////////////////////////////////
    /*
    Crea la PDA que guarda el Registro: TODOS los clientes del negocio y, dentro de cada
    cliente, TODO su historial de ventas. La PDA depende de:
        * Wallet address (owner)
        * Program ID
        * string representativo ("registro")

    Parametros de entrada:
        * nombre_negocio -> nombre del punto de venta -> String
     */
    pub fn crear_registro(context: Context<NuevoRegistro>, nombre_negocio: String) -> Result<()> {
        let owner_id = context.accounts.owner.key(); // Wallet address del caller

        let clientes = Vec::new(); // El negocio nace sin clientes
        let id_counter: u16 = 0; // Contador incremental, asigna un id unico a cada cliente nuevo

        context.accounts.registro.set_inner(Registro {
            owner: owner_id,
            nombre_negocio: nombre_negocio.clone(),
            clientes,
            id_counter,
        });

        msg!("Registro: {} creado exitosamente!", nombre_negocio);

        Ok(())
    }

    //////////////////////////// Instruccion: Registrar Venta /////////////////////////////////////
    /*
    Registra una venta. Dos comportamientos segun id_cliente:
        * id_cliente == 0 -> cliente nuevo: se incrementa el contador, se crea el Cliente y la
          venta queda como primer elemento de su historial.
        * id_cliente != 0 -> se busca al cliente en el vector y se le agrega la venta.

    NUEVO (aporte propio): el monto debe ser mayor a 0, si no falla con MontoInvalido.

    Parametros de entrada:
        * id_cliente  -> id del cliente, 0 = cliente nuevo -> u16
        * sku         -> codigo del producto vendido       -> String
        * descripcion -> descripcion de la venta           -> String
        * monto       -> importe de la venta               -> u32
     */
    pub fn registrar_venta(
        context: Context<ModificarVenta>,
        id_cliente: u16,
        sku: String,
        descripcion: String,
        monto: u32,
    ) -> Result<()> {
        require!(
            context.accounts.registro.owner == context.accounts.owner.key(),
            Errores::NoOwner
        );
        require!(monto > 0, Errores::MontoInvalido); // NUEVO: no se aceptan ventas de monto 0

        let registro = &mut context.accounts.registro;

        let venta = Venta {
            sku,
            descripcion,
            monto,
        };

        if id_cliente == 0 {
            // ---- Caso A: cliente nuevo ----
            registro.id_counter += 1;
            msg!(
                "Cliente creado con id: {}, del articulo {} registrada por un total de {}",
                registro.id_counter,
                venta.sku,
                venta.monto
            );
            let id = registro.id_counter;
            registro.clientes.push(Cliente {
                id_cliente: id,
                compras: 1,
                historial: vec![venta],
            });

            return Ok(());
        } else {
            // ---- Caso B: cliente existente ----
            for i in 0..registro.clientes.len() {
                if registro.clientes[i].id_cliente == id_cliente {
                    msg!(
                        "Compra del articulo {} registrada por un total de {}",
                        venta.sku,
                        venta.monto
                    );
                    registro.clientes[i].historial.push(venta);
                    registro.clientes[i].compras += 1;

                    return Ok(());
                }
            }
        }

        msg!("Cliente con id: {} no encontrado!", id_cliente);
        Err(Errores::NoUser.into())
    }

    //////////////////////////// Instruccion: Hacer Devolucion /////////////////////////////////////
    /*
    Elimina una venta del historial de un cliente a partir de su SKU.
    Error si el cliente no existe, si su historial esta vacio o si el SKU no aparece.

    Parametros de entrada:
        * id_cliente -> id del cliente que devuelve -> u16
        * sku        -> codigo del producto devuelto -> String
     */
    pub fn hacer_devolucion(
        context: Context<ModificarVenta>,
        id_cliente: u16,
        sku: String,
    ) -> Result<()> {
        require!(
            context.accounts.registro.owner == context.accounts.owner.key(),
            Errores::NoOwner
        );
        let clientes = &mut context.accounts.registro.clientes;

        for i in 0..clientes.len() {
            if clientes[i].id_cliente == id_cliente {
                require!(!clientes[i].historial.is_empty(), Errores::SinCompras);

                for j in 0..clientes[i].historial.len() {
                    if clientes[i].historial[j].sku == sku {
                        clientes[i].historial.remove(j);
                        clientes[i].compras -= 1;
                        msg!("Devolucion realizada exitosamente");
                        return Ok(());
                    }
                }

                return Err(Errores::NoSKU.into());
            }
        }
        Err(Errores::NoUser.into())
    }

    //////////////////////////// Instruccion: Cambiar Monto /////////////////////////////////////
    /*
    Corrige el importe de una venta ya registrada, localizandola por cliente y SKU.
    NUEVO (aporte propio): el nuevo monto debe ser mayor a 0.

    Parametros de entrada:
        * id_cliente   -> id del cliente      -> u16
        * sku          -> codigo del producto -> String
        * nuevo_monto  -> importe corregido   -> u32
     */
    pub fn cambiar_monto(
        context: Context<ModificarVenta>,
        id_cliente: u16,
        sku: String,
        nuevo_monto: u32,
    ) -> Result<()> {
        require!(
            context.accounts.registro.owner == context.accounts.owner.key(),
            Errores::NoOwner
        );
        require!(nuevo_monto > 0, Errores::MontoInvalido); // NUEVO

        let clientes = &mut context.accounts.registro.clientes;

        for i in 0..clientes.len() {
            if clientes[i].id_cliente == id_cliente {
                for j in 0..clientes[i].historial.len() {
                    if clientes[i].historial[j].sku == sku {
                        let viejo_monto = clientes[i].historial[j].monto;
                        clientes[i].historial[j].monto = nuevo_monto;
                        msg!(
                            "Monto actualizado existosamente, paso de {} a {}.",
                            viejo_monto,
                            nuevo_monto
                        );
                        return Ok(());
                    }
                }
                return Err(Errores::NoSKU.into());
            }
        }
        Err(Errores::NoUser.into())
    }

    //////////////////////////// Instruccion: Cantidad de Compras /////////////////////////////////////
    /*
    Muestra en el log de la transaccion el total de compras de un cliente.

    Una instruccion on-chain NO devuelve valores al frontend: msg! solo escribe en los logs.
    Para LEER datos, el cliente hace program.account.registro.fetch(pda).

    Parametros de entrada:
        * id_cliente -> id del cliente a consultar -> u16
     */
    pub fn cantidad_compras(context: Context<ModificarVenta>, id_cliente: u16) -> Result<()> {
        require!(
            context.accounts.registro.owner == context.accounts.owner.key(),
            Errores::NoOwner
        );

        let clientes = &context.accounts.registro.clientes;

        for i in 0..clientes.len() {
            if clientes[i].id_cliente == id_cliente {
                msg!(
                    "El cliente tiene un total de compras de {}:",
                    clientes[i].compras
                );
                msg!("Lista de productos comprados: {:#?}", clientes[i].historial);
                return Ok(());
            }
        }
        Err(Errores::NoUser.into())
    }

    //////////////////////////// Instruccion: Dar de Baja Cliente /////////////////////////////////////
    /*
    NUEVO. Elimina a un cliente del registro junto con todo su historial de ventas.
    Solo el owner puede hacerlo. Error NoUser si el cliente no existe.

    Parametros de entrada:
        * id_cliente -> id del cliente a eliminar -> u16
     */
    pub fn dar_baja_cliente(context: Context<ModificarVenta>, id_cliente: u16) -> Result<()> {
        require!(
            context.accounts.registro.owner == context.accounts.owner.key(),
            Errores::NoOwner
        );

        let clientes = &mut context.accounts.registro.clientes;

        // position() devuelve Some(indice) si encuentra al cliente, o None si no existe
        match clientes.iter().position(|c| c.id_cliente == id_cliente) {
            Some(i) => {
                clientes.remove(i); // Borra al cliente y, con el, todo su historial
                msg!("Cliente {} dado de baja exitosamente", id_cliente);
                Ok(())
            }
            None => Err(Errores::NoUser.into()),
        }
    }

    //////////////////////////// Instruccion: Cerrar Registro /////////////////////////////////////
    /*
    NUEVO (aporte propio). Cierra la cuenta Registro y devuelve la renta al owner, pero SOLO si
    ya no quedan clientes. Si todavia hay clientes, falla con RegistroConClientes y la cuenta
    no se toca. El cierre en si lo hace Anchor con "close = owner" en el contexto CerrarRegistro.
     */
    pub fn cerrar_registro(context: Context<CerrarRegistro>) -> Result<()> {
        require!(
            context.accounts.registro.clientes.is_empty(),
            Errores::RegistroConClientes
        );
        msg!("Registro cerrado y renta devuelta al owner");
        Ok(())
    }
}

/*
Codigos de error
Ojo con el nombre del enum: NO puede llamarse "Error", porque ese nombre ya lo ocupa
anchor_lang::prelude::Error (entra por el "use anchor_lang::prelude::*" de arriba).
*/
#[error_code]
pub enum Errores {
    #[msg("Error, no eres el owner del registro")]
    NoOwner,
    #[msg("Error, usuario no existe")]
    NoUser,
    #[msg("Error, sku no encontrado")]
    NoSKU,
    #[msg("Error, no hay compras en el historial")]
    SinCompras,
    #[msg("Error, el monto debe ser mayor a 0")]
    MontoInvalido,
    #[msg("Error, no se puede cerrar el registro porque aun tiene clientes")]
    RegistroConClientes,
}

#[account] // Cuenta que se almacena en la blockchain
#[derive(InitSpace)] // Genera INIT_SPACE: el espacio de almacenamiento necesario
pub struct Registro {
    owner: Pubkey,

    #[max_len(30)]
    nombre_negocio: String,

    #[max_len(10)] // Solo caben 10 clientes
    clientes: Vec<Cliente>,

    id_counter: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Debug)]
pub struct Cliente {
    id_cliente: u16,
    compras: u16,

    #[max_len(10)] // Solo 10 ventas POR CLIENTE
    historial: Vec<Venta>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Debug)]
pub struct Venta {
    #[max_len(5)]
    sku: String,

    #[max_len(50)]
    descripcion: String,

    monto: u32,
}

// Contexto de crear_registro
#[derive(Accounts)]
pub struct NuevoRegistro<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = Registro::INIT_SPACE + 8,
        seeds = [b"registro", owner.key().as_ref()],
        bump
    )]
    pub registro: Account<'info, Registro>,

    pub system_program: Program<'info, System>,
}

// Contexto compartido por las instrucciones que leen o modifican el registro
#[derive(Accounts)]
pub struct ModificarVenta<'info> {
    pub owner: Signer<'info>,

    #[account(mut)]
    pub registro: Account<'info, Registro>,
}

// NUEVO. Contexto de cerrar_registro: "close = owner" borra la cuenta y manda la renta al owner
#[derive(Accounts)]
pub struct CerrarRegistro<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, close = owner, has_one = owner)] // has_one: solo el owner del registro puede cerrarlo
    pub registro: Account<'info, Registro>,
}
