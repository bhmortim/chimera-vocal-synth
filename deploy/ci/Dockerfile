# chimera-ci: the pull-based CI/CD agent. The Docker CLI (with buildx and
# compose) drives the host daemon through the mounted socket; git and curl poll
# GitHub and check the deploy.
FROM docker:29.9-cli
RUN apk add --no-cache bash coreutils curl git \
 && docker buildx version && docker compose version
COPY agent.sh /usr/local/bin/chimera-ci
RUN chmod 0755 /usr/local/bin/chimera-ci
ENTRYPOINT ["/usr/local/bin/chimera-ci"]
