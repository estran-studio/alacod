# Serveur de signaling matchbox « nu » pour la CI de nuit (p2p headless, détection de desync).
# Le jeu natif ouvre un WebSocket matchbox standard (`ws://hôte:3536/<salle>`) ; allumette
# (dépôt frère) exige un jeton JWT dans le chemin, obtenu par son API HTTP que le jeu natif
# n'appelle pas encore : pour comparer les traces de N clients, matchbox_server suffit.
FROM rust:1.90-slim-bookworm AS builder
RUN cargo install matchbox_server --locked
FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/matchbox_server /usr/local/bin/matchbox_server
EXPOSE 3536
CMD ["matchbox_server", "0.0.0.0:3536"]
