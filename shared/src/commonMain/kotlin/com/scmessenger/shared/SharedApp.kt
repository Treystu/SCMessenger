package com.scmessenger.shared

expect fun platformName(): String

fun greet(): String = "Hello from ${platformName()}!"
