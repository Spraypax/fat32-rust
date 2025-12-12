# 🎯 Objectif : 
comprendre le fonctionnement interne de FAT32 sans dépendances externes, tout en respectant les contraintes systèmes de Rust.

## 🗂️ fat32-rust

 - Implémentation d’un lecteur FAT32 en Rust
 - Projet réalisé dans le cadre du cours Rust 4A – FAT32 reimplementation

---

## 📌 Présentation

**fat32-rust** est une implémentation pédagogique mais réaliste d’un lecteur **FAT32** en Rust.

Le projet est composé de :

 - 🧩 une **bibliothèque FAT32** (no_std + alloc)
 - 🖥️ un **CLI** (avec shell interactif) pour tester le système de fichiers sur une vraie image FAT32

---

## ✅ Fonctionnalités

### 📁 Système de fichiers / 

 - Parsing du **Boot Sector / BPB**
 - Calcul des offsets **FAT** et **zone data**
 - Lecture de la **FAT** (chaînes de clusters)
 - Conversion **cluster → LBA**
 - Gestion correcte de la racine **FAT32** (clusters < 2)

### 📂 Répertoires

- Lecture des entrées (short names 8.3)
- Ignorance des entrées supprimées et LFN
- Support :
    - chemins absolus (/DIR/FILE.TXT)
    - chemins relatifs (DIR/FILE.TXT)
    - `.` et `..`
- Gestion du répertoire courant (cd)

### 📄 Fichiers

- Lecture complète du contenu d’un fichier
- Lecture multi-secteurs / multi-clusters
- Commande cat fonctionnelle

---

## 🧱 Architecture du projet

```text
fat32-rust/
├── Cargo.lock
├── Cargo.toml
├── images
│   └── test_fat32.img
├── README.md
├── src
│   ├── bin
│   │   └── cli.rs
│   ├── boot.rs
│   ├── dir.rs
│   ├── fat.rs
│   ├── file.rs
│   └── lib.rs
└── tests
    └── fat32_basic.rs
```
🔒 La bibliothèque est no_std (avec alloc).
🧪 Le backend StdBlockDevice (std::fs::File) est utilisé uniquement pour le CLI et les tests.

## 🖥️ CLI

Un binaire cli est fourni pour tester le lecteur FAT32.

### ▶️ Commandes one-shot
```bash
cargo run --bin cli -- images/test_fat32.img ls /
cargo run --bin cli -- images/test_fat32.img cat /README.TXT
```
### 🐚 Shell interactif
```bash
cargo run --bin cli -- images/test_fat32.img shell
```

Commandes disponibles :

```bash
ls [path]
cd <path>
cat <path>
pwd
exit
```

## 🧪 Tests

Des tests d’intégration obligatoires sont fournis et couvrent :

- listing de la racine
- lecture de fichier
- cd vers un sous-répertoire
- chemins relatifs
- gestion de ..
- export CARGO_TARGET_DIR=$HOME/rust-target
- cargo test

## 🛠️ Qualité du code

Le projet suit une politique stricte de qualité :

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

✔️ Zéro warning Clippy
✔️ Aucun unsafe
✔️ Gestion explicite des erreurs
✔️ Code modulaire et lisible

## 🧠 Miri (Bonus)

Les tests ont été exécutés avec Miri.

Les tests utilisent StdBlockDevice, qui ouvre une vraie image disque FAT32.
Cela est interdit par défaut par l’isolation de Miri.

Les tests passent avec :

```bash
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test
```

👉 Cela confirme l’absence d’Undefined Behavior dans la logique du driver FAT32.

💽 Génération de l’image FAT32 de test
dd if=/dev/zero of=images/test_fat32.img bs=1M count=16
mkfs.vfat -F 32 images/test_fat32.img

```bash
sudo mkdir -p /mnt/fat32img
sudo mount -o loop images/test_fat32.img /mnt/fat32img

sudo bash -c 'echo "Hello from FAT32 root" > /mnt/fat32img/README.TXT'
sudo mkdir -p /mnt/fat32img/DIR1
sudo bash -c 'echo "Ceci est un fichier dans DIR1" > /mnt/fat32img/DIR1/FILE1.TXT'

sudo umount /mnt/fat32img
```

## 🏁 Conclusion

fat32-rust est une implémentation :

- complète
- propre
- testée
- conforme aux contraintes no_std
- avec outillage professionnel (clippy, miri, tests)
