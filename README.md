# Gestor de Claves CLI

Stack tecnológico:

Lenguaje: Rust (Seguridad de memoria garantizada).

Interfaz: CLI (Command Line Interface). para clave de desencriptación

Persistencia: JSON serializado.

Criptografía: ChaCha20Poly1305 (vía cocoon).

Visualización web a través de http://localhost:3000

- Debes tener rust instalado

# Para correr la app en terminal
- cargo run. (el archivo "mis_claves.db debe estar en la carpeta principal donde está la app, afuera de la carpeta 'src'")
- accede contraseña
- ingresa a http://localhost:3000

# Compilar la aplicación
- cargo build --release