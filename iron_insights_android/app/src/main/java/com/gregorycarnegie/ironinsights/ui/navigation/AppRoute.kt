package com.gregorycarnegie.ironinsights.ui.navigation

enum class AppRoute(
    val label: String,
    val navRoute: String,
) {
    LOOKUP("Lookup", NavRoutes.LOOKUP),
    COMPARE("Similar", NavRoutes.COMPARE),
    TRENDS("Trends", NavRoutes.TRENDS),
    CALCULATORS("Calc", NavRoutes.CALCULATORS),
    LOG("Log", NavRoutes.LOG),
    PROGRAMMES("Plan", NavRoutes.PROGRAMMES),
    PROGRESS("PRs", NavRoutes.PROGRESS),
    PROFILE("Profile", NavRoutes.PROFILE),
    ;

    val isAnalytics: Boolean
        get() = this in analyticsRoutes

    companion object {
        val analyticsRoutes = setOf(LOOKUP, COMPARE, TRENDS, CALCULATORS)

        fun fromNavRoute(route: String?): AppRoute? =
            entries.firstOrNull { it.navRoute == route }
    }
}
