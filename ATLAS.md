# Atlas

Ce dépôt est un fork de [Saber](https://github.com/saber-notes/saber) (GPL-3.0) : le cahier de
cours d'Atlas, second client de capture branché sur le serveur de Pensay.

- **Cadrage** : dépôt Pensay, `docs/ATLAS.md`.
- **Décisions techniques** : dépôt Pensay, `docs/DECISIONS.md` (sections « Appels aux modèles » et
  « Atlas : client de cours »).
- **STEP en cours** : dépôt Pensay, `docs/steps/STEP-A0.md`.

Le premier cadrage de ce dépôt (graphe de concepts, noyau Rust) est abandonné ; son code reste
dans l'historique git.

## Ce qui distingue le fork de Saber

| | Saber | Atlas |
|---|---|---|
| Identifiant Android | `com.adilhanney.saber` | `com.lumosity23.atlas` |
| Identifiant Linux | `com.adilhanney.saber` | `com.lumosity23.atlas` |
| Dossier des notes sur PC | `Documents/Saber` | `Documents/Atlas` |
| Synchro Nextcloud | active | coupée jusqu'à A2 |

Les deux apps s'installent côte à côte : installer, réinstaller ou désinstaller Atlas ne touche
jamais Saber ni ses notes.

Le code propre à Atlas vit dans `lib/atlas/`. Les fichiers de Saber ne reçoivent que des points de
branchement courts, marqués `Atlas:` ou importés depuis `lib/atlas/`.

## Compiler

Flutter **3.47.8** (la version épinglée par Saber dans `submodules/flutter`).

```bash
flutter pub get
flutter analyze
flutter test
```

APK pour la tablette (sur une machine avec le SDK Android) :

```bash
flutter build apk --release
```

Sans `android/key.properties`, l'APK est signé avec la clé de secours de Saber
(`android/fallback-key.properties`), publique puisqu'elle est dans le dépôt. Ça suffit pour tester.
Pour l'usage quotidien, crée ta propre clé et un `android/key.properties` (non versionné) qui la
désigne : avec une clé publique, n'importe qui pourrait signer une mise à jour de ton app.

**Sonde du double tap du stylet** (A0, partie C) : un APK qui affiche en bas de l'écran le type
d'événement reçu quand on tape deux fois sur le stylet. Aucun contenu de note n'est affiché ni
journalisé.

```bash
flutter build apk --release --dart-define=ATLAS_STYLUS_PROBE=true
```

### Tests de Saber qui échouent hors de son environnement

Ces tests échouent aussi sur Saber non modifié, dans un conteneur Linux :
- les comparaisons d'images (`*_golden_test`, `home_theme_test`, `glassy_container_test`,
  `screenshot_goldens_test`) : le rendu des polices diffère ; Saber les lance dans le conteneur de
  `test.sh` ;
- `nc_upload_download_test` et `nc_deletion_test` : ils parlent au vrai serveur Nextcloud de Saber.

## Récupérer les mises à jour de Saber

Le fork est un instantané de Saber (l'historique complet pèse 1,1 Go). Le commit de Saber importé
est noté dans le message du commit d'import, puis dans chaque commit de mise à jour.

```bash
git remote add saber https://github.com/saber-notes/saber.git   # une seule fois
git fetch saber main
git diff <commit-saber-importé> saber/main -- . | git apply -3
```

`git apply -3` fusionne à trois voies : un conflit n'apparaît que là où Atlas et Saber ont modifié
les mêmes lignes.
