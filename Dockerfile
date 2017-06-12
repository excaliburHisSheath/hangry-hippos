FROM ubuntu:12.04

EXPOSE 80 6768 6769

ENV ROCKET_ENV=staging

CMD ["target/debug/hangry-hippos"]
