# Atlas — architecture et feuille de route

Objectif : un « cerveau de connaissances » personnel, agent-first. Tout ce que tu sais,
notes, exercices, dessins et conversations avec des agents IA y est relié, et un agent
peut lire et écrire dans le même espace que toi.

## Principes (repris de LearnHarness)

1. **Pas de score de maîtrise stocké.** Ce que tu sais est *dérivé* des preuves
   (`Evidence` avec `verified = true`). Lire une note ne valide rien.
2. **Les liens sont explicites.** Une arête `prerequisite` est vérifiée à l'écriture
   (pas de cycle). Les autres arêtes décrivent l'association.
3. **Ajout seulement pour les preuves.** `evidence.jsonl` ne se réécrit jamais.
   Le reste (`concepts.json`, `edges.json`) est l'état courant.
4. **Fichiers lisibles.** Un vault est un dossier de JSON/JSONL, inspectable et versionnable.
5. **Agent-first.** Chaque commande lisible accepte `--json`. Le même noyau servira
   plus tard à un serveur MCP pour les agents.

## Structure

```
atlas/
├── crates/
│   ├── atlas-core/   noyau pur : modèle, graphe, persistance fichiers, plan d'étude
│   └── atlas-cli/    binaire `atlas`, interface agent/humain en ligne de commande
└── ARCHITECTURE.md
```

Vault par défaut : `./atlas-vault` (ou `$ATLAS_VAULT`).

## Modèle

| Type | Rôle |
|---|---|
| `Concept` | une unité de compréhension (titre, résumé, tags) |
| `Edge` | lien `prerequisite` / `related` / `contrast` / `example` |
| `Evidence` | trace d'une interaction : note, exercice, conversation, dessin ; `verified` = démontré par toi |

Vues dérivées : `known()`, `plan(goal)`, `context(concept)`.

## Feuille de route

- **Phase 1 (faite)** : noyau du graphe, preuves, plan d'étude, CLI, tests.
- **Phase 2 : serveur MCP** au-dessus d'`atlas-core`, pour qu'un agent (Claude, etc.)
  lise le graphe, propose un plan, enregistre des preuves après une conversation.
- **Phase 3 : couche notes et dessin.** Base sur le moteur de rnote (`rnote-compose`,
  `rnote-engine`) : strokes, pages, export. Les pages deviennent des `Source::Note`.
- **Phase 4 : capture papier.** Photo d'une page → transcription/vision par agent →
  `Evidence` liée aux concepts.
- **Phase 5 : interfaces.** Desktop (Linux, en premier) et tablette/téléphone.
- **Phase 6 : synchronisation** entre appareils, multi-agents, mode « prof » qui
  planifie les leçons depuis le graphe.

## Points de décision ouverts

- **Licence.** rnote est GPL-3.0-or-later, LearnHarness est AGPL-3.0. Réutiliser du code
  de rnote impose une licence compatible GPL-3 pour l'ensemble. Le projet est donc
  placé en AGPL-3.0-or-later, ce qui est compatible. Ne pas copier de code de LearnHarness
  sans accord sur cette licence : le projet s'en inspire, il ne le réutilise pas.
- **Interface tablette/téléphone.** rnote-ui est GTK4/libadwaita, donc Linux uniquement.
  Le moteur (`rnote-engine`) est, lui, réutilisable. Pour mobile il faudra une autre
  couche (Tauri, Flutter ou PWA avec noyau Rust en WASM).

## Décisions prises (session de cadrage)

- **Serveur** : Debian (VPS), source de vérité du vault et du noyau Rust.
- **Accès** : sous-domaine Cloudflare via **Cloudflare Tunnel** (aucun port ouvert). Protection
  en amont par Cloudflare Access. Tailscale reste envisageable pour l'administration seulement.
- **Agent** : API, fournisseur non figé (Claude n'est pas imposé). Le code d'appel doit
  passer par une interface unique, pour changer de modèle sans toucher au reste.
- **Tablette** : APK Android. À confirmer : modèle exact (« Tab S3 » ; la Galaxy Tab S3 est
  un appareil Samsung avec S Pen) et version d'Android, qui limite la version minimale de l'app.
- **Hors-ligne obligatoire** : la tablette garde une copie locale (base locale + fichiers
  d'encre) et se synchronise avec le serveur au retour du réseau.
- **Conception du front** : discutée avant le code. Voir `docs/brainstorm-prompt.md`.
