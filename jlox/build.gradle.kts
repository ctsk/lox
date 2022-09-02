plugins {
    id("application")
}

group = "xyz.ctsk"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation("org.junit.jupiter:junit-jupiter-api:5.9.0")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:5.9.0")
}

application {
    mainClass.set("xyz.ctsk.jlox.Hello")
}


tasks.getByName<Test>("test") {
    useJUnitPlatform()
}