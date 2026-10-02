package me.kavishdevar.librepods.data.app

import kotlinx.serialization.Serializable

@Serializable
data class FontSettings (
    val scale: FontSize = FontSize.Medium,
    val fontFamilyOption: FontFamilyOption = FontFamilyOption.RobotoFlexExpressive
)

enum class FontSize {
    VerySmall,
    Small,
    Medium,
    Large,
    VeryLarge
}

enum class FontFamilyOption {
    SystemDefault,
    RobotoFlex,
    RobotoFlexExpressive,
    Inter
}
