FROM ubuntu:24.04 as rust

RUN apt-get update && apt-get install -y curl cmake g++

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN chmod +x $HOME/.cargo/env

RUN $HOME/.cargo/bin/cargo --version

WORKDIR producer
COPY producer .

RUN $HOME/.cargo/bin/cargo build

FROM eclipse-temurin:25.0.1_8-jre as builder
WORKDIR application
ARG JAR_FILE=build/libs/*.jar
COPY ${JAR_FILE} application.jar
RUN java -Djarmode=layertools -jar application.jar extract

FROM eclipse-temurin:25.0.1_8-jre
WORKDIR application
COPY --from=builder application/dependencies/ ./
COPY --from=builder application/spring-boot-loader/ ./
COPY --from=builder application/snapshot-dependencies/ ./
COPY --from=builder application/application/ ./
COPY --from=rust producer/target/debug/libproducer* ./

RUN ls -al

ENTRYPOINT ["java", "-Djava.library.path=.", "org.springframework.boot.loader.JarLauncher"]
