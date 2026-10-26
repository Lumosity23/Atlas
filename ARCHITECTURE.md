# Atlas — état du cadrage

**Le cadrage d'Atlas vit désormais dans le dépôt Pensay : `docs/ATLAS.md`.**

Décision du 2026-10-26 : Atlas n'a pas son propre serveur. C'est un second client de capture
(un fork de Saber, pour écrire au stylet sur tablette, PC et téléphone) branché sur le serveur de
Pensay. Pensay capture les pensées, Atlas capture les cours, le serveur relie les deux.

## Ce qui est abandonné dans ce dépôt

Le premier cadrage venait d'un agent, pas d'un besoin exprimé. Il est remplacé :

| Abandonné | Remplacé par |
|---|---|
| Graphe de concepts avec prérequis vérifiés | des notions reliées aux pages, dans le moteur d'entités de Pensay |
| Preuves `verified` et « acquis seulement si démontré » | des fiches de niveau lisibles et corrigeables, écrites par le seul rôle Prof |
| Plan d'étude automatique, multi-agents, répétition espacée | un seul agent avec des outils neutres et des fiches de rôle |
| Noyau Rust (`crates/`) et CLI | le backend FastAPI de Pensay |
| rnote comme moteur d'encre, APK Android seul | fork de Saber (Flutter) : Android, Windows, Linux |

Le code de `crates/` reste en place tant que son sort n'est pas tranché ; plus aucun travail ne
s'y fait.
