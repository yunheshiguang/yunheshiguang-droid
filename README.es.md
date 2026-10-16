# yunheshiguang

[English](./README.md) · [العربية](./README.ar.md) · [中文](./README.zh-CN.md) · [Français](./README.fr.md) · [Русский](./README.ru.md) · **Español**

Un banco de trabajo de escritorio para operar flotas de dispositivos Android desde una sola computadora: conectarlos, verlos y manejarlos por lotes. Construido sobre Tauri 2 (backend en Rust, interfaz en React). Los dispositivos se conectan por ADB, sin instalar nada en ellos.

## Funciones

- **Muro de dispositivos**: USB, Wi-Fi (emparejamiento y descubrimiento mDNS) y teléfonos en la nube se conectan en un mismo lugar. El servidor ADB notifica cada cambio de dispositivo, sin sondeos. Los dispositivos se pueden numerar, nombrar, etiquetar y agrupar. Los números de serie y demás datos sensibles se ocultan por defecto, y los valores ocultos no se pueden buscar.
- **Operaciones por lotes**: captura de pantalla, grabación de pantalla, instalar un APK, subir un archivo, introducir texto, escribir en el portapapeles, iniciar o detener una app, desinstalar, borrar datos, recuperar la conexión y ejecutar un script. Las acciones irreversibles, como desinstalar o borrar datos, piden confirmación antes.
- **Duplicación de pantalla**: abre una ventana por dispositivo con [scrcpy](https://github.com/Genymobile/scrcpy) oficial, descargado desde la app con una versión y un resumen SHA-256 fijados.
- **Automatización**: editor de flujos, plantillas iniciales, programaciones cron e historial de ejecuciones. Los flujos se ejecutan en el backend; la interfaz solo los inicia, los detiene y muestra su progreso.
- **Scripts**: una consola de scripts de shell que devuelve la salida y el código de salida de cada dispositivo.
- **Borradores con IA**: describe un flujo o un script en una frase y la IA redacta un borrador que se muestra mientras se escribe. Los borradores solo se abren en el editor; guardar y ejecutar siempre depende de ti. Funciona con cualquier API compatible con OpenAI (OpenAI, DeepSeek, Qwen, un Ollama local y otras); las claves se quedan en el backend de esta computadora.
- **Gobernanza e informes**: cada acción registra a su operador. El registro de auditoría se guarda en una base de datos SQLite local y se puede filtrar y exportar como CSV o JSON Lines. Los informes suman las operaciones por grupo, operador, acción y día.
- **Paquetes de plantillas**: comparte flujos y scripts como paquetes con un resumen de su contenido, opcionalmente firmados con [minisign](https://jedisct1.github.io/minisign/). La vista previa de instalación muestra quién los firmó.
- **API local**: HTTP y WebSocket para scripts y otros sistemas. Desactivada por defecto.
- **Interfaz**: chino e inglés, temas claro y oscuro.

## Sistemas compatibles

| Sistema | ADB del sistema o personalizado | Descarga de Platform Tools | Descarga de scrcpy |
| --- | --- | --- | --- |
| Windows x86-64 | Sí | Sí | Sí |
| Windows ARM64 | Sí | No | Sí |
| macOS (Intel, Apple silicon) | Sí | Sí | Sí |
| Linux x86-64 (glibc) | Sí | Sí | Sí |
| Otros Linux | Con un ADB compatible | No | No |

La app descarga las Android Platform Tools 37.0.0 oficiales después de que aceptes la licencia del SDK de Android. Las URL, los tamaños y los resúmenes SHA-256 están fijados en el código fuente; un paquete que no coincide se rechaza antes de extraerlo.

## Primeros pasos

Requiere Node.js 24, pnpm 10, Rust 1.98.1 (lo selecciona `rust-toolchain.toml`) y las [dependencias del sistema de Tauri](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm install
pnpm tauri dev      # ejecutar la app de escritorio
pnpm tauri build    # generar los instaladores
```

Para trabajar solo en la interfaz, ejecuta `pnpm dev` y abre <http://localhost:1420> en un navegador. Sin un host de Tauri, implementaciones en memoria de los mismos servicios Connect responden a la interfaz; las compilaciones de producción no las incluyen.

## Descargas y actualizaciones

Los instaladores están en la [página de versiones](https://github.com/yunheshiguang/yunheshiguang-android/releases):

- **Estable**: la última versión, publicada a mano a partir de una etiqueta de versión.
- **Nightly**: la versión preliminar `nightly`, que se vuelve a generar cada vez que un cambio en `main` pasa la CI.

Todas las versiones usan los mismos nombres de archivo (`yunheshiguang-windows-x86_64-setup.exe`, `yunheshiguang-macos-arm64.dmg`, `yunheshiguang-macos-x86_64.dmg`, `yunheshiguang-linux-x86_64.AppImage`, `yunheshiguang-linux-x86_64.deb`, `yunheshiguang-linux-x86_64.rpm`), junto con `SHA256SUMS` y `release.json`. La aplicación revisa su propio canal una vez al día (**Ajustes → Actualizaciones**) y enlaza el archivo que le corresponde cuando hay una compilación más reciente; nunca descarga ni instala nada por su cuenta. Las compilaciones todavía no están firmadas con un certificado de desarrollador. [docs/releases.md](./docs/releases.md) describe los canales, el manifiesto y cómo publicar una versión.

## API local

Actívala en **Settings → Local API**. Solo escucha en `127.0.0.1`, en el puerto 47831 por defecto. Los tokens se emiten en la misma página; cada token se muestra una sola vez y la app solo guarda su resumen. La API sirve los mismos servicios Connect que las ventanas de la app, a clientes Connect, gRPC y gRPC-Web por igual. Todas las solicitudes salvo `/v1/health` necesitan un token:

```sh
curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{}' \
  http://127.0.0.1:47831/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot
```

`/v1/openapi.json` describe todos los procedimientos que sirve la API (OpenAPI 3.1, generado a partir de los servicios de `proto/`); los procedimientos que solo leen datos también responden a `GET`. Un procedimiento se sirve por la API cuando su definición en `proto/` lleva la opción `local_api`. `/v1/events` es un WebSocket que envía los eventos del espacio de trabajo; los navegadores, que no pueden ponerle cabeceras, pueden usar `?access_token=` en su lugar. Las acciones hechas a través de la API pasan por la misma autorización y la misma auditoría que las ventanas de la app, y cada cliente tiene su propio límite de solicitudes.

## Extensiones

La compilación por defecto es el núcleo de código abierto y no contiene código de extensiones. Una extensión tiene dos partes: `native`, un crate de plugin de Tauri que solo depende de `crates/sdk` y `crates/contract`, y `web`, un paquete de frontend que usa el host solo a través de `yunheshiguang/sdk`. Las extensiones pueden aportar autorización, identidad, licencias, almacenamiento y destinos adicionales de auditoría, almacenamiento de documentos, fuentes de plantillas, métodos de conexión, telemetría, adaptadores de fabricante, canales de entrada, motores de scripts, autenticación de la API, proveedores de IA y proveedores de teléfonos en la nube; lo que ninguna extensión aporta usa la implementación por defecto del núcleo.

`extensions/reference` es la extensión de referencia e implementa la mayoría (una política de autorización Cedar, identidad de equipo, una licencia, una auditoría compartida por el equipo, almacenamiento en una carpeta sincronizada, un mercado de plantillas, teléfonos en la nube Redroid, un proveedor de IA para API nativas como Claude y Gemini, métricas de Prometheus, un adaptador para granjas de placas, entrada con ADBKeyBoard, el motor de scripts AutoJs6, claves de API empresariales y una página de empresa):

```sh
pnpm tauri dev --config src-tauri/tauri.reference.conf.json   # ejecutar con la extensión de referencia
pnpm dev --mode reference                                      # solo la interfaz
```

## Desarrollo

| Comando | Qué hace |
| --- | --- |
| `pnpm test` | Biome, el límite de longitud de archivos, las capas FSD (Steiger), los límites de importación, Knip, el contrato entre frontend y backend, y Vitest |
| `pnpm build` | Comprueba los tipos y compila el frontend |
| `pnpm check:distributions` | Compila la versión de código abierto y cada distribución compuesta, y confirma que la de código abierto no lleva código de extensiones |
| `pnpm proto:lint` / `pnpm proto:generate` / `pnpm proto:openapi` / `pnpm proto:verify` | Revisa los contratos, regenera los enlaces de TypeScript, regenera el documento OpenAPI y comprueba que el documento versionado está al día |
| `cargo test --workspace --all-features` | Pruebas de Rust, incluida la compilación compuesta con extensiones |
| `pnpm lint:rust` / `pnpm lint:deps` | clippy (`-D warnings`) y cargo-deny |

Los contratos de `proto/` son la única fuente de verdad: `pnpm install` genera los enlaces de TypeScript en `src/shared/proto/gen/` (no se versionan) y los de Rust se generan durante la compilación.

## Componentes de terceros

- [YADB](https://github.com/ysbing/YADB) v1.1.2 (LGPL-3.0): escribe texto y el portapapeles del dispositivo; incluido sin modificar en `src-tauri/assets/yadb`.
- [Android SDK Platform Tools](https://developer.android.com/tools/releases/platform-tools): se descargan bajo demanda tras aceptar su licencia; no se distribuyen con la app.
- [scrcpy](https://github.com/Genymobile/scrcpy) (Apache-2.0): sus versiones oficiales se descargan bajo demanda; no se distribuye con la app.

## Licencia

[GPL-3.0-only](./LICENSE)
