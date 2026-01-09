use serde::{Serialize, Deserialize};
use cocoon::Cocoon;
use std::error::Error;
use std::fs::File;

// Agregamos 'pub' para que sean accesibles desde fuera
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entrada {
    pub servicio: String, // 'pub' aquí también para poder leerlo
    pub usuario: String,
    pub clave: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Boveda {
    pub entradas: Vec<Entrada>,
}

impl Boveda {
    pub fn nueva() -> Boveda {
        Boveda { entradas: Vec::new() }
    }

    pub fn agregar(&mut self, entrada: Entrada) {
        self.entradas.push(entrada);
    }

    // Nota: Eliminé 'pub' de los campos internos de Boveda para obligar a usar estos métodos
    
    pub fn guardar(&self, nombre_archivo: &str, clave_maestra: &str) -> Result<(), Box<dyn Error>> {
        let mut cocoon = Cocoon::new(clave_maestra.as_bytes());
        let datos_serializados = serde_json::to_vec(&self.entradas)?;
        let mut archivo = File::create(nombre_archivo)?;
        cocoon.dump(datos_serializados, &mut archivo).unwrap();
        Ok(())
    }

    pub fn cargar(nombre_archivo: &str, clave_maestra: &str) -> Result<Boveda, Box<dyn Error>> {
        let cocoon = Cocoon::new(clave_maestra.as_bytes());
        let mut archivo = File::open(nombre_archivo)?;
        let datos_desencriptados = cocoon.parse(&mut archivo)
            .map_err(|e| format!("Error de encriptación: {:?}", e))?;
        let entradas: Vec<Entrada> = serde_json::from_slice(&datos_desencriptados)?;
        Ok(Boveda { entradas })
    }

    pub fn eliminar(&mut self, index: usize) -> Result<(), String> {
        if index < self.entradas.len() {
            self.entradas.remove(index);
            Ok(())
        } else {
            Err("El número de contraseña no existe".to_string())
        }
    }

    pub fn editar(&mut self, index: usize, nueva_clave: String) -> Result<(), String> {
        if index < self.entradas.len() {
            self.entradas[index].clave = nueva_clave;
            Ok(())
        } else {
            Err("Índice inválido".to_string())
        }
    }
}