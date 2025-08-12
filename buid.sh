#!/bin/bash

# Script para compilar las diferentes versiones de Freqdash

echo "🔨 Compilando Freqdash..."

# CLI version
echo "📦 Compilando versión CLI..."
cargo build --release --features cli --bin cli

# GTK4 version
echo "📦 Compilando versión GTK4..."
cargo build --release --features gtk --bin gui

# Adwaita version
echo "📦 Compilando versión Adwaita..."
cargo build --release --features adwaita --bin gui_adw

echo "✅ Compilación completada!"
echo ""
echo "Binarios disponibles en target/release/:"
echo "  - cli      : Versión de línea de comandos"
echo "  - gui      : Versión GTK4 tradicional"
echo "  - gui_adw  : Versión con libadwaita (moderna)"