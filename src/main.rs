mod boveda;
use boveda::{Boveda, Entrada};

use axum::{
    extract::{State, Form},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use serde::Deserialize; // Para entender los datos que vienen del HTML
use std::sync::{Arc, Mutex}; // Herramientas para compartir memoria de forma segura
use std::net::SocketAddr;
use std::io::{self, Write};

// 1. ESTRUCTURA DEL ESTADO GLOBAL
// Guardamos la Bóveda (que cambia) y la Clave Maestra (que es fija)
struct EstadoApp {
    boveda: Mutex<Boveda>,
    clave_maestra: String,
}

// Usamos Arc para compartir este estado entre hilos
type EstadoCompartido = Arc<EstadoApp>;

// 2. ESTRUCTURA PARA EL FORMULARIO
// Los nombres de los campos deben coincidir con el "name" en el HTML
#[derive(Deserialize)]
struct DatosFormulario {
    servicio: String,
    usuario: String,
    clave: String,
}

// 🆕 NUEVA ESTRUCTURA PARA BORRAR
#[derive(Deserialize)]
struct DatosBorrar {
    indice: usize,
}

#[tokio::main]
async fn main() {
    let nombre_archivo = "mis_claves.db";
    
    println!("--- 🌐 SERVIDOR WEB DE CLAVES ---");
    print!("🔑 Introduce tu Contraseña Maestra: ");
    io::stdout().flush().unwrap();
    let password = rpassword::read_password().unwrap();

    let boveda_cargada = match Boveda::cargar(nombre_archivo, &password) {
        Ok(b) => {
            println!("✅ Bóveda cargada.");
            b
        },
        Err(_) => {
            println!("⚠️ Iniciando bóveda nueva.");
            Boveda::nueva()
        }
    };

    let estado = Arc::new(EstadoApp {
        boveda: Mutex::new(boveda_cargada),
        clave_maestra: password,
    });

    let app = Router::new()
        .route("/", get(ver_lista))
        .route("/guardar", post(guardar_clave))
        .route("/borrar", post(borrar_clave)) // 🆕 NUEVA RUTA
        .with_state(estado);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Servidor listo en: http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- HANDLERS ---

async fn ver_lista(State(estado): State<EstadoCompartido>) -> Html<String> {
    let boveda = estado.boveda.lock().unwrap();

    let mut html = String::from(r#"
        <!DOCTYPE html>
        <html lang="es">
        <head>
            <meta charset="UTF-8">
            <title>Bóveda Rust</title>
            <style>
                body { font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; background: #f0f2f5; }
                h1 { color: #1a73e8; text-align: center; }
                
                .form-box { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); margin-bottom: 20px; }
                input { width: 100%; padding: 10px; margin: 5px 0 15px 0; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box; }
                button { background-color: #1a73e8; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; width: 100%; font-size: 16px; }
                button:hover { background-color: #1557b0; }

                /* Tarjetas */
                .card { background: white; padding: 15px; margin-bottom: 10px; border-radius: 8px; border-left: 5px solid #1a73e8; box-shadow: 0 1px 3px rgba(0,0,0,0.1); display: flex; justify-content: space-between; align-items: center; }
                .info { flex-grow: 1; }
                .servicio { font-weight: bold; font-size: 1.1em; }
                .datos { color: #555; margin-top: 5px; font-size: 0.9em; }
                .clave { font-family: monospace; background: #eee; padding: 2px 6px; border-radius: 4px; color: #d93025; }
                
                /* Botón Borrar */
                .btn-borrar { background-color: #e53935; padding: 8px 15px; font-size: 14px; width: auto; margin-left: 10px; }
                .btn-borrar:hover { background-color: #b71c1c; }
            </style>
        </head>
        <body>
            <h1>🔐 Bóveda Digital Rust</h1>

            <div class="form-box">
                <h3>➕ Agregar Nueva Contraseña</h3>
                <form action="/guardar" method="post">
                    <label>Servicio:</label>
                    <input type="text" name="servicio" placeholder="Ej. Netflix" required>
                    <label>Usuario / Email:</label>
                    <input type="text" name="usuario" placeholder="Ej. correo@gmail.com" required>
                    <label>Contraseña:</label>
                    <input type="text" name="clave" placeholder="Tu contraseña segura" required>
                    <button type="submit">Guardar en Bóveda</button>
                </form>
            </div>

            <h3>📂 Tus Cuentas Guardadas</h3>
    "#);

    if boveda.entradas.is_empty() {
        html.push_str("<p style='text-align:center; color:#777;'><i>La bóveda está vacía.</i></p>");
    } else {
        // Usamos enumerate() para saber el ÍNDICE (0, 1, 2...) de cada tarjeta
        // Nota: Quitamos el .rev() para simplificar el borrado y que los índices coincidan
        for (indice, entrada) in boveda.entradas.iter().enumerate() {
            let tarjeta = format!(
                r#"
                <div class="card">
                    <div class="info">
                        <div class="servicio">{}</div>
                        <div class="datos">Usuario: <b>{}</b> | Clave: <span class="clave">{}</span></div>
                    </div>
                    
                    <form action="/borrar" method="post" onsubmit="return confirm('¿Seguro que quieres borrar esta cuenta?');">
                        <input type="hidden" name="indice" value="{}">
                        <button type="submit" class="btn-borrar">🗑️ Borrar</button>
                    </form>
                </div>
                "#,
                entrada.servicio, entrada.usuario, entrada.clave, indice // Pasamos el índice aquí
            );
            html.push_str(&tarjeta);
        }
    }
    html.push_str("</body></html>");
    Html(html)
}

async fn guardar_clave(
    State(estado): State<EstadoCompartido>, 
    Form(datos): Form<DatosFormulario>
) -> Redirect {
    let mut boveda = estado.boveda.lock().unwrap();
    let nueva_entrada = Entrada {
        servicio: datos.servicio,
        usuario: datos.usuario,
        clave: datos.clave,
    };
    boveda.agregar(nueva_entrada);
    
    if let Err(e) = boveda.guardar("mis_claves.db", &estado.clave_maestra) {
        println!("❌ Error guardando: {}", e);
    }
    Redirect::to("/")
}

// 🆕 HANDLER PARA BORRAR
async fn borrar_clave(
    State(estado): State<EstadoCompartido>,
    Form(datos): Form<DatosBorrar>
) -> Redirect {
    let mut boveda = estado.boveda.lock().unwrap();

    // Intentamos eliminar usando el método que ya creaste en boveda.rs
    match boveda.eliminar(datos.indice) {
        Ok(_) => {
            println!("🗑️ Elemento {} eliminado.", datos.indice);
            // Guardamos cambios en disco
            if let Err(e) = boveda.guardar("mis_claves.db", &estado.clave_maestra) {
                println!("❌ Error guardando tras borrar: {}", e);
            }
        },
        Err(e) => println!("❌ No se pudo borrar: {}", e),
    }

    Redirect::to("/")
}