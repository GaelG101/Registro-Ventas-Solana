import { AnchorProvider, Program, BN, type Idl } from '@coral-xyz/anchor'
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js'
import type { AnchorWallet } from '@solana/wallet-adapter-react'
import idl from './idl/idl.json'

export const PROGRAM_ID = new PublicKey('788k8AagUMgf4zhJ519UxsF6QduZmfdFr4okVxURbWhg')
export const connection = new Connection('https://api.devnet.solana.com', 'confirmed')

export function getProgram(wallet: AnchorWallet) {
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' })
  return new Program(idl as Idl, PROGRAM_ID, provider)
}

export function getRegistroPda(owner: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('registro'), owner.toBuffer()],
    PROGRAM_ID
  )[0]
}

export async function crearRegistro(wallet: AnchorWallet, nombreNegocio: string) {
  const program = getProgram(wallet)
  return program.methods
    .crearRegistro(nombreNegocio)
    .accounts({
      owner: wallet.publicKey,
      registro: getRegistroPda(wallet.publicKey),
      systemProgram: SystemProgram.programId,
    })
    .rpc()
}

export async function registrarVenta(
  wallet: AnchorWallet,
  idCliente: number,
  sku: string,
  descripcion: string,
  monto: number
) {
  const program = getProgram(wallet)
  return program.methods
    .registrarVenta(idCliente, sku, descripcion, monto)
    .accounts({
      owner: wallet.publicKey,
      registro: getRegistroPda(wallet.publicKey),
    })
    .rpc()
}

export async function leerRegistro(wallet: AnchorWallet) {
  const program = getProgram(wallet)
  return (program.account as any).registro.fetch(getRegistroPda(wallet.publicKey))
}

export { BN }
