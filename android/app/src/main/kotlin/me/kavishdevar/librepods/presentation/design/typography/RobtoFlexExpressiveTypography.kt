package me.kavishdevar.librepods.presentation.design.typography

import androidx.compose.material3.Typography
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import me.kavishdevar.librepods.R

private fun robotoFlex(
    wght: Float = 400f,
    slnt: Float = 0f,
    grad: Float = 0f,
    wdth: Float = 100f,
    xtra: Float = 468f,
    xopq: Float = 96f,
    yopq: Float = 79f,
) = FontFamily(
    Font(
        resId = R.font.roboto_flex,
        variationSettings = FontVariation.Settings(
            FontVariation.Setting("wght", wght),
            FontVariation.Setting("wdth", wdth),
            FontVariation.Setting("slnt", slnt),
            FontVariation.Setting("grad", grad),
            FontVariation.Setting("xtra", xtra),
            FontVariation.Setting("xopq", xopq),
            FontVariation.Setting("yopq", yopq),
        )
    )
)

val display = robotoFlex(
    wght = 800f,
    grad = 100f,
    wdth = 100f
)

val displayEmphasized = robotoFlex(
    wght = 1000f,
    slnt = -2f,
    grad = 150f,
    wdth = 150f,
)

val body = robotoFlex()

val bodyEmphasized = robotoFlex(
    wght = 600f,
    wdth = 130f,
    grad = 75f,
)

val label = robotoFlex(
    wght = 450f,
    grad = 50f
)

val labelEmphasized = robotoFlex(
    wght = 600f,
    wdth = 140f,
    grad = 75f
)


val RobotoFlexExpressiveTypography = Typography().run {
    copy(
        titleSmall = titleSmall.copy(fontFamily = display),
        titleMedium = titleMedium.copy(fontFamily = display),
        titleLarge = titleLarge.copy(fontFamily = display),

        titleSmallEmphasized = titleSmallEmphasized.copy(fontFamily = displayEmphasized),
        titleMediumEmphasized = titleMediumEmphasized.copy(fontFamily = displayEmphasized),
        titleLargeEmphasized = titleLargeEmphasized.copy(fontFamily = displayEmphasized),

        displaySmall = displaySmall.copy(fontFamily = display),
        displayMedium = displayMedium.copy(fontFamily = display),
        displayLarge = displayLarge.copy(fontFamily = display),

        displaySmallEmphasized = displaySmallEmphasized.copy(fontFamily = displayEmphasized),
        displayMediumEmphasized = displayMediumEmphasized.copy(fontFamily = displayEmphasized),
        displayLargeEmphasized = displayLargeEmphasized.copy(fontFamily = displayEmphasized),

        bodySmall = bodySmall.copy(fontFamily = body),
        bodyMedium = bodyMedium.copy(fontFamily = body),
        bodyLarge = bodyLarge.copy(fontFamily = body),

        bodySmallEmphasized = bodySmallEmphasized.copy(fontFamily = bodyEmphasized),
        bodyMediumEmphasized = bodyMediumEmphasized.copy(fontFamily = bodyEmphasized),
        bodyLargeEmphasized = bodyLargeEmphasized.copy(fontFamily = bodyEmphasized),

        labelSmall = labelSmall.copy(fontFamily = label),
        labelMedium = labelMedium.copy(fontFamily = label),
        labelLarge = labelLarge.copy(fontFamily = label),

        labelSmallEmphasized = labelSmallEmphasized.copy(fontFamily = labelEmphasized),
        labelMediumEmphasized = labelMediumEmphasized.copy(fontFamily = labelEmphasized),
        labelLargeEmphasized = labelLargeEmphasized.copy(fontFamily = labelEmphasized)
    )
}
