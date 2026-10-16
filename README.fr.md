# yunheshiguang

[English](./README.md) · [العربية](./README.ar.md) · [中文](./README.zh-CN.md) · **Français** · [Русский](./README.ru.md) · [Español](./README.es.md)

Un poste de travail de bureau pour piloter des parcs d'appareils Android depuis un seul ordinateur : les connecter, les observer et les commander par lots. Construit sur Tauri 2 (backend Rust, interface React). Les appareils se connectent via ADB, sans rien installer dessus.

## Fonctionnalités

- **Mur d'appareils** : USB, Wi-Fi (appairage et découverte mDNS) et téléphones cloud se connectent au même endroit. Le serveur ADB signale lui-même chaque changement d'appareil, sans interrogation périodique. Les appareils peuvent être numérotés, nommés, étiquetés et regroupés. Les numéros de série et les autres données sensibles sont masqués par défaut, et les valeurs masquées ne sont pas recherchables.
- **Opérations par lots** : capture d'écran, enregistrement de l'écran, installation d'APK, envoi de fichier, saisie de texte, écriture du presse-papiers, lancement ou arrêt d'une application, désinstallation, effacement des données, reconnexion et exécution de script. Les actions irréversibles, comme la désinstallation ou l'effacement des données, demandent d'abord une confirmation.
- **Recopie d'écran** : ouvre une fenêtre par appareil avec [scrcpy](https://github.com/Genymobile/scrcpy) officiel, téléchargé depuis l'application avec une version et une empreinte SHA-256 figées.
- **Automatisation** : éditeur de flux, modèles de départ, planifications cron et historique des exécutions. Les flux s'exécutent dans le backend ; l'interface se contente de les lancer, de les arrêter et d'afficher leur progression.
- **Scripts** : une console de scripts shell qui renvoie la sortie et le code de sortie de chaque appareil.
- **Brouillons par IA** : décrivez un flux ou un script en une phrase, et l'IA en rédige un brouillon qui s'affiche au fil de l'écriture. Les brouillons s'ouvrent uniquement dans l'éditeur ; l'enregistrement et l'exécution restent toujours entre vos mains. Fonctionne avec toute API compatible OpenAI (OpenAI, DeepSeek, Qwen, un Ollama local, etc.) ; les clés restent dans le backend de cet ordinateur.
- **Gouvernance et rapports** : chaque action enregistre son opérateur. Le journal d'audit est conservé dans une base SQLite locale, filtrable et exportable en CSV ou en JSON Lines. Les rapports totalisent les opérations par groupe, opérateur, action et jour.
- **Paquets de modèles** : partagez des flux et des scripts sous forme de paquets dotés d'une empreinte de contenu, éventuellement signés avec [minisign](https://jedisct1.github.io/minisign/). L'aperçu avant installation indique le signataire.
- **API locale** : HTTP et WebSocket pour les scripts et les autres systèmes. Désactivée par défaut.
- **Interface** : chinois et anglais, thèmes clair et sombre.

## Systèmes pris en charge

| Système | ADB du système ou personnalisé | Téléchargement de Platform Tools | Téléchargement de scrcpy |
| --- | --- | --- | --- |
| Windows x86-64 | Oui | Oui | Oui |
| Windows ARM64 | Oui | Non | Oui |
| macOS (Intel, Apple silicon) | Oui | Oui | Oui |
| Linux x86-64 (glibc) | Oui | Oui | Oui |
| Autres Linux | Avec un ADB compatible | Non | Non |

L'application télécharge les Android Platform Tools 37.0.0 officiels une fois la licence du SDK Android acceptée. Les URL, les tailles et les empreintes SHA-256 sont figées dans le code source ; un paquet qui ne correspond pas est rejeté avant extraction.

## Démarrage

Prérequis : Node.js 24, pnpm 10, Rust 1.98.1 (sélectionné par `rust-toolchain.toml`) et les [dépendances système de Tauri](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm install
pnpm tauri dev      # lancer l'application de bureau
pnpm tauri build    # générer les installateurs
```

Pour ne travailler que sur l'interface, lancez `pnpm dev` et ouvrez <http://localhost:1420> dans un navigateur. Sans hôte Tauri, des implémentations en mémoire des mêmes services Connect répondent à l'interface ; les builds de production ne les incluent pas.

## Téléchargements et mises à jour

Les installateurs se trouvent sur la [page des versions](https://github.com/yunheshiguang/yunheshiguang-android/releases) :

- **Stable** : la dernière version, publiée à la main à partir d’une étiquette de version.
- **Nightly** : la préversion `nightly`, reconstruite chaque fois qu’une modification sur `main` passe la CI.

Chaque version porte les mêmes noms de fichiers (`yunheshiguang-windows-x86_64-setup.exe`, `yunheshiguang-macos-arm64.dmg`, `yunheshiguang-macos-x86_64.dmg`, `yunheshiguang-linux-x86_64.AppImage`, `yunheshiguang-linux-x86_64.deb`, `yunheshiguang-linux-x86_64.rpm`), accompagnés de `SHA256SUMS` et de `release.json`. L’application consulte son propre canal une fois par jour (**Paramètres → Mises à jour**) et propose un lien vers le fichier qui lui correspond quand une version plus récente est disponible ; elle ne télécharge ni n’installe jamais rien d’elle-même. Les versions ne sont pas encore signées avec un certificat de développeur. [docs/releases.md](./docs/releases.md) décrit les canaux, le manifeste et la publication d’une version.

## API locale

Activez-la dans **Settings → Local API**. Elle n'écoute que sur `127.0.0.1`, sur le port 47831 par défaut. Les jetons s'émettent sur la même page ; chaque jeton n'est affiché qu'une seule fois et l'application n'en conserve que l'empreinte. L'API sert les mêmes services Connect que les fenêtres de l'application, aux clients Connect, gRPC et gRPC-Web. Toute requête, sauf `/v1/health`, exige un jeton :

```sh
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{}' \
  http://127.0.0.1:47831/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot
```

`/v1/openapi.json` décrit toutes les procédures que sert l'API (OpenAPI 3.1, généré à partir des services de `proto/`) ; les procédures qui ne font que lire acceptent aussi `GET`. Une procédure est servie par l'API lorsque sa définition dans `proto/` porte l'option `local_api`. `/v1/events` est un WebSocket qui diffuse les événements de l'espace de travail ; les navigateurs, qui ne peuvent pas y définir d'en-têtes, peuvent passer `?access_token=` à la place. Les actions effectuées via l'API passent par les mêmes autorisations et le même audit que les fenêtres de l'application, et chaque appelant a sa propre limite de débit.

## Extensions

Le build par défaut est le cœur open source et ne contient aucun code d'extension. Une extension comporte deux parties : `native`, un crate de plugin Tauri qui ne dépend que de `crates/sdk` et `crates/contract`, et `web`, un paquet frontend qui n'accède à l'hôte que par `yunheshiguang/sdk`. Les extensions peuvent fournir l'autorisation, l'identité, les licences, le stockage et les destinations supplémentaires de l'audit, le stockage des documents, les sources de modèles, les méthodes de connexion, la télémétrie, les adaptateurs de fabricant, les canaux de saisie, les moteurs de scripts, l'authentification de l'API, les fournisseurs d'IA et les fournisseurs de téléphones cloud ; tout ce qu'aucune extension ne fournit utilise l'implémentation par défaut du cœur.

`extensions/reference` est l'extension de référence et en implémente la plupart (une politique d'autorisation Cedar, l'identité d'équipe, une licence, un audit partagé par l'équipe, le stockage dans un dossier synchronisé, une place de marché de modèles, des téléphones cloud Redroid, un fournisseur d'IA pour les API natives comme Claude et Gemini, des métriques Prometheus, un adaptateur pour fermes de cartes, la saisie par ADBKeyBoard, le moteur de scripts AutoJs6, des clés d'API d'entreprise et une page entreprise) :

```sh
pnpm tauri dev --config src-tauri/tauri.reference.conf.json   # lancer avec l'extension de référence
pnpm dev --mode reference                                      # interface seule
```

## Développement

| Commande | Rôle |
| --- | --- |
| `pnpm test` | Biome, la limite de longueur des fichiers, les couches FSD (Steiger), les frontières d'import, Knip, le contrat entre frontend et backend, et Vitest |
| `pnpm build` | Vérifie les types et construit le frontend |
| `pnpm check:distributions` | Construit la version open source et chaque distribution composée, et vérifie que la version open source ne contient aucun code d'extension |
| `pnpm proto:lint` / `pnpm proto:generate` / `pnpm proto:openapi` / `pnpm proto:verify` | Vérifie les contrats, régénère les liaisons TypeScript, régénère le document OpenAPI et contrôle que le document versionné est à jour |
| `cargo test --workspace --all-features` | Tests Rust, y compris le build composé avec les extensions |
| `pnpm lint:rust` / `pnpm lint:deps` | clippy (`-D warnings`) et cargo-deny |

Les contrats de `proto/` sont la seule source de vérité : `pnpm install` génère les liaisons TypeScript dans `src/shared/proto/gen/` (non versionnées), et les liaisons Rust sont générées au moment du build.

## Composants tiers

- [YADB](https://github.com/ysbing/YADB) v1.1.2 (LGPL-3.0) : saisit du texte et écrit le presse-papiers de l'appareil ; intégré sans modification dans `src-tauri/assets/yadb`.
- [Android SDK Platform Tools](https://developer.android.com/tools/releases/platform-tools) : téléchargés à la demande après acceptation de leur licence ; non distribués avec l'application.
- [scrcpy](https://github.com/Genymobile/scrcpy) (Apache-2.0) : versions officielles téléchargées à la demande ; non distribué avec l'application.

## Licence

[GPL-3.0-only](./LICENSE)
