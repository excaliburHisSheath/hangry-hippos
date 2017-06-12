FROM ubuntu:16.04

ADD . /hangry-river-horse

EXPOSE 80 6768 6769

ENV ROCKET_ENV=staging

WORKDIR /hangry-river-horse
CMD ["bin/hangry-river-horse"]
