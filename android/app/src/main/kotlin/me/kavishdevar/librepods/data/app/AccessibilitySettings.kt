package me.kavishdevar.librepods.data.app

import kotlinx.serialization.Serializable

@Serializable
data class AccessibilitySettings(
    val differentiateWithoutColor: Boolean = false
)
