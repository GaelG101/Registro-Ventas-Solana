import { useState } from 'react'
import { useAnchorWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { crearRegistro, registrarVenta, leerRegistro, getRegistroPda } from './client'

export default function App() {
  const wallet = useAnchorWallet()
  const [negocio, setNegocio] = useState('')
  const [idCliente, setIdCliente] = useState('1')
  const [sku, setSku] = useState('')
  const [descripcion, setDescripcion] = useState('')
  const [monto, setMonto] = useState('')
  const [sig, setSig] = useState('')
  const [estado, setEstado] = useState('')
  const [datos, setDatos] = useState('')

  async function ejecutar(accion: () => Promise<string>) {
    try {
      setEstado('Firmando y enviando...')
      const firma = await accion()
      setSig(firma)
      setEstado('Confirmada')
    } catch (e: any) {
      setEstado('Error: ' + (e?.message ?? String(e)))
    }
  }

  async function ver() {
    if (!wallet) return
    try {
      const r = await leerRegistro(wallet)
      setDatos(JSON.stringify(r, null, 2))
    } catch (e: any) {
      setDatos('No se pudo leer: ' + (e?.message ?? String(e)))
    }
  }

  return (
    <div style={{ maxWidth: 560, margin: '0 auto', padding: 24, textAlign: 'left' }}>
      <h1>Registro de Ventas (Devnet)</h1>
      <WalletMultiButton />
      {wallet && <p>Tu registro (PDA): {getRegistroPda(wallet.publicKey).toBase58()}</p>}

      <h2>1. Crear registro</h2>
      <input placeholder="Nombre del negocio" value={negocio} onChange={e => setNegocio(e.target.value)} />
      <button disabled={!wallet} onClick={() => ejecutar(() => crearRegistro(wallet!, negocio))}>Crear registro</button>

      <h2>2. Registrar venta</h2>
      <input placeholder="ID cliente" value={idCliente} onChange={e => setIdCliente(e.target.value)} />
      <input placeholder="SKU" value={sku} onChange={e => setSku(e.target.value)} />
      <input placeholder="Descripción" value={descripcion} onChange={e => setDescripcion(e.target.value)} />
      <input placeholder="Monto" value={monto} onChange={e => setMonto(e.target.value)} />
      <button disabled={!wallet} onClick={() => ejecutar(() => registrarVenta(wallet!, Number(idCliente), sku, descripcion, Number(monto)))}>Registrar venta</button>

      <h2>3. Ver registro</h2>
      <button disabled={!wallet} onClick={ver}>Leer datos</button>
      <pre style={{ whiteSpace: 'pre-wrap' }}>{datos}</pre>

      <p>{estado}</p>
      {sig && (
        <p>
          <a href={`https://explorer.solana.com/tx/${sig}?cluster=devnet`} target="_blank">Ver transacción en Explorer</a>
        </p>
      )}
    </div>
  )
}
