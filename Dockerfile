FROM ubuntu:22.04 as rust

RUN /bin/bash -c "$(curl -fsSL https://apt.kitware.com/kitware-archive.sh)"
RUN apt-get update && apt-get install -y cmake

RUN /bin/bash -c "$(curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y)"

RUN /bin/bash -c "$(source $HOME/.cargo/env && cargo --version)"

WORKDIR producer
COPY producer .

RUN /bin/bash -c "$(source $HOME/.cargo/env && cargo build)"

FROM eclipse-temurin:19-jre as builder
WORKDIR application
ARG JAR_FILE=build/libs/*.jar
COPY ${JAR_FILE} application.jar
RUN java -Djarmode=layertools -jar application.jar extract

FROM eclipse-temurin:19-jre
WORKDIR application
COPY --from=builder application/dependencies/ ./
COPY --from=builder application/spring-boot-loader/ ./
COPY --from=builder application/snapshot-dependencies/ ./
COPY --from=builder application/application/ ./
COPY --from=rust producer/target/debug/ ./

RUN ls -al

ENTRYPOINT ["java", "-Djava.library.path=.", "org.springframework.boot.loader.JarLauncher"]
