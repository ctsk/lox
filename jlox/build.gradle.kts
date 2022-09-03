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
    mainClass.set("xyz.ctsk.lox.Lox")
}

tasks.jar {
    manifest {
        attributes(mapOf("Main-Class" to "xyz.ctsk.lox.Lox"))
    }
}


tasks.getByName<Test>("test") {
    useJUnitPlatform()
}

tasks.named<JavaExec>("run") {
    standardInput = System.`in`
}