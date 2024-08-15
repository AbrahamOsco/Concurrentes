#!/bin/bash

# Ancho y alto de la pantalla
SCREEN_WIDTH=$(xdpyinfo | awk '/dimensions/{print $2}' | cut -d 'x' -f 1)
SCREEN_HEIGHT=$(xdpyinfo | awk '/dimensions/{print $2}' | cut -d 'x' -f 2)

# Tamaño de las terminales
TERM_WIDTH=$(($SCREEN_WIDTH / 3))
TERM_HEIGHT=$(($SCREEN_HEIGHT / 3))

# Función para abrir una nueva terminal en una posición específica
open_terminal() {
    local CMD=$1
    local X_POS=$2
    local Y_POS=$3
    gnome-terminal --geometry=80x24+$X_POS+$Y_POS -- bash -c "$CMD; exec bash"
}

# Abre los 3 primeros comandos en una fila
open_terminal "cargo run --bin gateway 2" 0 0
open_terminal "cargo run --bin gateway 1" $TERM_WIDTH 0
open_terminal "cargo run --bin gateway 0" $(($TERM_WIDTH * 2)) 0

# Abre los siguientes 4 comandos en una fila
open_terminal "cargo run --bin order" 0 $TERM_HEIGHT
open_terminal "cargo run --bin robot 2" $(($SCREEN_WIDTH / 4)) $TERM_HEIGHT
open_terminal "cargo run --bin robot 1" $(($SCREEN_WIDTH / 2)) $TERM_HEIGHT
open_terminal "cargo run --bin robot 0" $(($SCREEN_WIDTH * 3 / 4)) $TERM_HEIGHT

# Abre los últimos 4 comandos en una fila
open_terminal "cargo run --bin repo" 0 $(($TERM_HEIGHT * 2))
open_terminal "cargo run --bin interface 0" $(($SCREEN_WIDTH / 4)) $(($TERM_HEIGHT * 2))
open_terminal "cargo run --bin interface 1" $(($SCREEN_WIDTH / 2)) $(($TERM_HEIGHT * 2))
open_terminal "cargo run --bin interface 2" $(($SCREEN_WIDTH * 3 / 4)) $(($TERM_HEIGHT * 2))
