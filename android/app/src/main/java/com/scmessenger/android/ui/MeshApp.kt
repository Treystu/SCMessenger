package com.scmessenger.android.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.Chat
import androidx.compose.material.icons.filled.People
import androidx.compose.material.icons.filled.Router
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.scmessenger.android.ui.contacts.AddContactScreen
import com.scmessenger.android.ui.identity.IdentityScreen
import com.scmessenger.android.ui.screens.*
import com.scmessenger.android.ui.viewmodels.MainViewModel

/**
 * Root composable for the SCMessenger app.
 *
 * Sets up the navigation graph and bottom navigation bar.
 */
@Composable
fun MeshApp() {
    val mainViewModel: MainViewModel = hiltViewModel()
    val hasIdentity by mainViewModel.hasIdentity.collectAsState()
    val showOnboarding by mainViewModel.showOnboarding.collectAsState()
    val isStorageLow by mainViewModel.isStorageLow.collectAsState()
    val availableStorageMB by mainViewModel.availableStorageMB.collectAsState()
    val navController = rememberNavController()

    LaunchedEffect(Unit) {
        mainViewModel.refreshIdentityState()
    }

    LaunchedEffect(hasIdentity) {
        val currentRoute = navController.currentBackStackEntry?.destination?.route
        val allowedRoutes = roleBasedBottomNavItems(hasIdentity).map { it.route }.toSet()
        if (currentRoute != null && currentRoute !in allowedRoutes && !currentRoute.startsWith("chat/")) {
            navController.navigate(startDestinationForRole(hasIdentity)) {
                launchSingleTop = true
            }
        }
    }

    Column(modifier = Modifier.fillMaxSize()) {
        if (isStorageLow) {
            com.scmessenger.android.ui.components.StorageWarningBanner(availableMB = availableStorageMB)
        }

        if (showOnboarding) {
            OnboardingScreen(
                onOnboardingComplete = { mainViewModel.refreshIdentityState() },
                viewModel = mainViewModel,
                modifier = Modifier.weight(1f)
            )
        } else {
            Scaffold(
                modifier = Modifier.weight(1f),
                bottomBar = { MeshBottomBar(navController = navController, hasIdentity = hasIdentity) }
            ) { paddingValues ->
                MeshNavHost(
                    navController = navController,
                    hasIdentity = hasIdentity,
                    onIdentityChanged = { mainViewModel.refreshIdentityState() },
                    bottomPadding = paddingValues
                )
            }
        }
    }
}

@Composable
fun MeshNavHost(
    navController: NavHostController,
    hasIdentity: Boolean,
    onIdentityChanged: () -> Unit,
    bottomPadding: PaddingValues = PaddingValues()
) {
    NavHost(
        navController = navController,
        startDestination = startDestinationForRole(hasIdentity)
    ) {
        if (hasIdentity) {
            composable(Screen.Conversations.route) {
                ConversationsScreen(
                    onNavigateToChat = { peerId ->
                        navController.navigate("chat/$peerId")
                    }
                )
            }

            composable(Screen.Contacts.route) {
                ContactsScreen(
                    onNavigateToChat = { peerId ->
                        navController.navigate("chat/$peerId")
                    },
                    onNavigateToAddContact = {
                        navController.navigate(Screen.AddContact.route)
                    }
                )
            }

            composable(Screen.AddContact.route) {
                AddContactScreen(
                    onNavigateBack = { navController.popBackStack() },
                    onContactAdded = { navController.popBackStack() }
                )
            }
        }

        composable(Screen.Dashboard.route) {
            DashboardScreen()
        }

        composable(Screen.Settings.route) {
            Box(modifier = Modifier.padding(bottomPadding)) {
                SettingsScreen(
                    onNavigateToIdentity = {
                        navController.navigate(Screen.Identity.route)
                    },
                    onNavigateToDiagnostics = {
                        navController.navigate(Screen.Diagnostics.route)
                    },
                    onNavigateToBlockedPeers = {
                        navController.navigate(Screen.BlockedPeers.route)
                    }
                )
            }
        }

        composable(Screen.Identity.route) {
            IdentityScreen(
                onNavigateBack = {
                    onIdentityChanged()
                    navController.popBackStack()
                }
            )
        }

        composable(Screen.Diagnostics.route) {
            DiagnosticsScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable(Screen.BlockedPeers.route) {
            BlockedPeersScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        if (hasIdentity) {
            composable(
                route = "chat/{peerId}",
                arguments = listOf(androidx.navigation.navArgument("peerId") { type = androidx.navigation.NavType.StringType })
            ) { backStackEntry ->
                val peerId = backStackEntry.arguments?.getString("peerId") ?: return@composable
                ChatScreen(
                    conversationId = peerId,
                    onNavigateBack = { navController.popBackStack() }
                )
            }
        }
    }
}

/**
 * Bottom navigation bar.
 */
@Composable
fun MeshBottomBar(navController: NavHostController, hasIdentity: Boolean) {
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route

    // Hide bottom bar on Chat screen
    if (currentRoute?.startsWith("chat/") == true) return

    NavigationBar {
        roleBasedBottomNavItems(hasIdentity).forEach { screen ->
            NavigationBarItem(
                icon = { Icon(screen.icon, contentDescription = screen.label) },
                label = { Text(screen.label) },
                selected = currentRoute == screen.route,
                onClick = {
                    navController.navigate(screen.route) {
                        popUpTo(navController.graph.startDestinationId) {
                            saveState = true
                        }
                        launchSingleTop = true
                        restoreState = true
                    }
                }
            )
        }
    }
}

/**
 * Screen definitions for navigation.
 */
sealed class Screen(val route: String, val label: String, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    object Conversations : Screen("conversations", "Chats", androidx.compose.material.icons.Icons.Default.Chat)
    object Contacts : Screen("contacts", "Contacts", androidx.compose.material.icons.Icons.Default.People)
    object AddContact : Screen("add_contact", "Add Contact", androidx.compose.material.icons.Icons.Filled.Add)
    object Dashboard: Screen("dashboard", "Mesh", androidx.compose.material.icons.Icons.Filled.Router)
    object Settings : Screen("settings", "Settings", androidx.compose.material.icons.Icons.Default.Settings)
    object Identity : Screen("identity", "Identity", androidx.compose.material.icons.Icons.Default.Settings)
    object Diagnostics : Screen("diagnostics", "Diagnostics", androidx.compose.material.icons.Icons.Default.Settings)
    object BlockedPeers : Screen("blocked_peers", "Blocked Peers", androidx.compose.material.icons.Icons.Filled.Block)

    companion object {
        val fullRoleBottomNavItems = listOf(Conversations, Contacts, Dashboard, Settings)
        val relayOnlyBottomNavItems = listOf(Dashboard, Settings)
    }
}

internal fun roleBasedBottomNavItems(hasIdentity: Boolean): List<Screen> =
    if (hasIdentity) Screen.fullRoleBottomNavItems else Screen.relayOnlyBottomNavItems

internal fun startDestinationForRole(hasIdentity: Boolean): String =
    if (hasIdentity) Screen.Conversations.route else Screen.Dashboard.route
