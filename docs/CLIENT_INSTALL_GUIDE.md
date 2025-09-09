# 🚀 Portify - Instalación para Usuarios

**Portify** es una herramienta que te permite ver y gestionar los puertos activos en tu Mac directamente desde la barra de menú.

## 📋 Requisitos del Sistema

- **macOS 10.13** o superior
- **Python 3.8+** (se instala automáticamente si no lo tienes)

## 🎯 Instalación Rápida (Recomendada)

### **Paso 1: Abrir Terminal**

1. Presiona `Cmd + Espacio`
2. Escribe "Terminal" y presiona Enter

### **Paso 2: Ejecutar Instalador**

Copia y pega este comando en Terminal:

```bash
curl -sSL https://raw.githubusercontent.com/tu-usuario/portify/main/install-menubar.sh | bash
```

### **Paso 3: ¡Listo!**

- Verás el ícono "P" en tu barra de menú
- Haz clic para ver tus puertos activos
- Clic en ❌ para cerrar procesos

## 🔧 Instalación Manual

Si prefieres instalar paso a paso:

### **1. Instalar Python (si no lo tienes)**

```bash
# Verificar si tienes Python
python3 --version

# Si no tienes Python, descárgalo de:
# https://python.org/downloads/
```

### **2. Descargar Portify**

```bash
# Descargar desde GitHub
git clone https://github.com/tu-usuario/portify.git
cd portify
```

### **3. Instalar Portify**

```bash
# Ejecutar instalador
./install-menubar.sh
```

## 🎯 Cómo Usar Portify

### **Interfaz de Barra de Menú**

1. **Busca el ícono "P"** en tu barra de menú (arriba a la derecha)
2. **Haz clic** para ver el menú desplegable
3. **Ve tus puertos activos** con información detallada
4. **Mata procesos** haciendo clic en ❌

### **Comandos de Terminal (Opcional)**

```bash
# Ver todos los puertos
portify list

# Solo puertos en escucha
portify list --listening

# Filtrar por aplicación
portify list --filter chrome

# Matar un proceso
portify kill 1234

# Abrir barra de menú
portify menubar
```

## 🎨 Características

- 🎯 **Siempre visible** - Ícono en barra de menú
- ⚡ **Un clic para matar** - Cierra procesos instantáneamente
- 🔄 **Auto-actualización** - Se actualiza cada 5 segundos
- 🎨 **Códigos de color** - Estados visuales claros
- 🔔 **Notificaciones** - Alertas del sistema
- 📊 **Información detallada** - Puerto, protocolo, estado

### **Colores del Ícono**

- 🔵 **Azul**: Funcionamiento normal
- 🟢 **Verde**: Puertos activos escuchando
- 🟡 **Amarillo**: Alta actividad
- 🔴 **Rojo**: Problemas detectados
- ⚫ **Gris**: Sin puertos activos

## 🆘 Solución de Problemas

### **No veo el ícono "P"**

1. Verifica que Portify esté ejecutándose: `ps aux | grep portify`
2. Reinicia la aplicación: `portify menubar`
3. Revisa los permisos en Preferencias del Sistema

### **"Acceso Denegado" al matar procesos**

1. Ve a **Preferencias del Sistema** > **Seguridad y Privacidad**
2. Otorga **Acceso Completo al Disco** a Terminal
3. Para procesos del sistema, usa `sudo portify kill <PID>`

### **Errores de instalación**

1. Verifica tu conexión a internet
2. Asegúrate de tener permisos de administrador
3. Intenta la instalación manual

### **El ícono aparece en el Dock**

Esto es normal durante la instalación. La versión final solo aparece en la barra de menú.

## 🔄 Desinstalación

Si necesitas desinstalar Portify:

```bash
# Detener la aplicación
pkill -f portify

# Desinstalar el paquete
pip uninstall portify

# Limpiar archivos (opcional)
rm -rf ~/.portify
```

## 📞 Soporte

### **¿Necesitas ayuda?**

- 📖 **Documentación completa**: [GitHub Repository](https://github.com/tu-usuario/portify)
- 🐛 **Reportar problemas**: [GitHub Issues](https://github.com/tu-usuario/portify/issues)
- 💬 **Comunidad**: [Discussions](https://github.com/tu-usuario/portify/discussions)

### **Comandos de ayuda**

```bash
portify --help          # Ayuda general
portify list --help     # Ayuda para listar puertos
portify menubar --help  # Ayuda para barra de menú
```

---

## 🎉 ¡Disfruta de Portify!

Una vez instalado, tendrás control total sobre los puertos y procesos de tu Mac directamente desde la barra de menú. ¡Perfecto para desarrolladores y administradores de sistemas!

**¿Problemas?** No dudes en contactarnos a través de GitHub Issues.
