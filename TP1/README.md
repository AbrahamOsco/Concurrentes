# TP1 - Concurrentes (Leer)

1. Estoy usando rust-rover y me tira errores si trabajo con la carpeta que se obtiene al clonar el repo
   "2024_1c_tp1_abraham_osco" asi que hay que cambiarle de nombre de "2024_1c_tp1_abraham_osco" a
   "a_2024_1c_tp1_abraham_osco" para poder ejecutar el proyecto con exito. El error:
    ```
   error: invalid character `2` in package name: `2024_1c_tp1_abraham_osco`, the name cannot start with a digit
    --> Cargo.toml:2:8
    |
    2 | name = "2024_1c_tp1_abraham_osco"
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
   ```
1. El proyecto se ejecuta con exito solo quiero justiicar la decision que tome por el error anterior.
2. Supuestos:
    1. Si me ingresan una cantidad de threads mayor a la cantidad de archivos en data entonces la cantidad de threads a
       lanzar sera igual a la cantidad de archivos jsonl en la carpeta data.
    2. El tamanio de cada chunk se calculara como la division entre la cantidad de archivos .jsonl y la cantidad de
       threads el resultado de esta division se truncara hacia abajo (floor), obteniendo chunks de menor tamanio y por
       lo tanto lanzaremos mas threads.


1. [![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/VqwN-ppG)
