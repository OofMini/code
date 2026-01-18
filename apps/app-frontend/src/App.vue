<script setup>
import { AuthFeature, PanelVersionFeature, TauriModrinthClient } from '@modrinth/api-client'
import {
    ArrowBigUpDashIcon,
    ChangeSkinIcon,
    CompassIcon,
    DownloadIcon,
    ExternalIcon,
    HomeIcon,
    LeftArrowIcon,
    LibraryIcon,
    LogInIcon,
    LogOutIcon,
    MaximizeIcon,
    MinimizeIcon,
    NewspaperIcon,
    NotepadTextIcon,
    PlusIcon,
    RefreshCwIcon,
    RestoreIcon,
    RightArrowIcon,
    ServerIcon,
    SettingsIcon,
    UserIcon,
    WorldIcon,
    XIcon,
} from '@modrinth/assets'
import {
    Admonition,
    Avatar,
    Button,
    ButtonStyled,
    commonMessages,
    defineMessages,
    NewsArticleCard,
    NotificationPanel,
    OverflowMenu,
    ProgressSpinner,
    provideModrinthClient,
    provideNotificationManager,
    providePageContext,
    useDebugLogger,
    useVIntl,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import { type } from '@tauri-apps/plugin-os'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { computed, defineAsyncComponent, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from './components/ui/ContextMenu.vue'
import UpdateToast from './components/ui/UpdateToast.vue'
import { initAuth } from './helpers/auth'
import { handleDeepLink } from './helpers/import'
import { initLogs } from './helpers/logs'
import { useTheme } from './store/theme'

// Removed PromotionWrapper import for lightweight/ad-free build
// Removed initAnalytics import

const InstanceCreationModal = defineAsyncComponent(
    () => import('./components/ui/InstanceCreationModal.vue')
)

const AuthGrantFlowWaitModal = defineAsyncComponent(
    () => import('./components/ui/modal/AuthGrantFlowWaitModal.vue')
)

const router = useRouter()
const route = useRoute()
const { t } = useI18n()

// ... (Meta tag logic preserved) ...

// Removed initAnalytics() call

onMounted(async () => {
    await initAuth()
    await initLogs()

    // ... (Deep link and event listeners preserved) ...
    const unlisten = await getCurrentWindow().onCloseRequested(async () => {
        await saveWindowState(StateFlags.ALL)
    });

    // ... (Existing mount logic) ...
})

// ... (Rest of script setup logic for navigation/client setup) ...
const client = new TauriModrinthClient({
    token: localStorage.getItem('token') || undefined,
    baseURL: import.meta.env.VITE_API_URL,
    version: '0.0.0', // This would be dynamic in production
});
provideModrinthClient(client);

</script>

<template>
    <div
        id="app"
        class="h-full w-full bg-background-base text-content-base overflow-hidden flex flex-col font-sans select-none"
        :class="useTheme().themeClass"
    >
        <ContextMenu />
        <UpdateToast />
        <InstanceCreationModal />
        <AuthGrantFlowWaitModal />

        <router-view />

    </div>
</template>

<style lang="scss">
@import './assets/stylesheets/global.scss';

/* Toast Animations */
.toast-enter-active {
    transition: opacity 0.25s linear;
}

.toast-enter-from,
.toast-leave-to {
    opacity: 0;
}

@media (prefers-reduced-motion: no-preference) {
    .toast-enter-active,
    .nav-button-animated-enter-active {
        transition: all 0.5s cubic-bezier(0.15, 1.4, 0.64, 0.96);
    }

    .toast-leave-active,
    .nav-button-animated-leave-active {
        transition: all 0.25s ease;
    }

    .toast-enter-from {
        scale: 0.5;
        translate: 0 -10rem;
        opacity: 0;
    }

    .toast-leave-to {
        scale: 0.96;
        translate: 20rem 0;
        opacity: 0;
    }

    .nav-button-animated-enter-active {
        position: relative;
    }

    .nav-button-animated-enter-active::before {
        content: '';
        inset: 0;
        border-radius: 100vw;
        background-color: var(--color-brand-highlight);
        position: absolute;
        animation: pop 0.5s ease-in forwards;
        opacity: 0;
    }

    @keyframes pop {
        0% {
            scale: 0.5;
        }
        50% {
            opacity: 0.5;
        }
        100% {
            scale: 1.5;
            opacity: 0;
        }
    }
}
</style>
