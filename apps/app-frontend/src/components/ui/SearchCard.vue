<script setup lang="ts">
import { useVIntl } from '@modrinth/ui';
import { useRouter } from 'vue-router';
import { openUrl } from '@tauri-apps/plugin-opener';
import { DownloadIcon } from '@modrinth/assets';

const props = defineProps({
    project: {
        type: Object,
        required: true
    },
    instance: {
        type: Object,
        default: null
    },
    categories: {
        type: Array,
        default: () => []
    },
    installed: {
        type: Boolean,
        default: false
    }
});

const emit = defineEmits(['install']);

const router = useRouter();
const { formatCompactNumber } = useVIntl();

const handleClick = () => {
    // Intercept CurseForge clicks
    if (props.project.__source === 'curseforge') {
        const url = `https://www.curseforge.com/minecraft/mc-mods/${props.project.slug}`;
        openUrl(url);
        return;
    }

    // Default Modrinth behavior
    router.push({
        name: 'project',
        params: {
            id: props.project.slug,
            type: props.project.project_type
        }
    });
};

const handleInstall = (e: Event) => {
    e.stopPropagation();
    emit('install', props.project.project_id);
};
</script>

<template>
    <div 
        class="search-card bg-background-element hover:bg-background-element-hover transition-colors rounded-lg p-3 cursor-pointer flex gap-4 h-32 w-full"
        @click="handleClick"
    >
        <img 
            :src="project.icon_url || 'https://cdn.modrinth.com/placeholder.svg'" 
            class="h-24 w-24 rounded-md object-cover bg-background-base shrink-0"
            alt=""
        />

        <div class="flex flex-col flex-grow min-w-0 justify-between py-1">
            <div>
                <div class="flex justify-between items-start gap-2">
                    <h3 class="font-bold text-lg leading-tight truncate pr-2" :title="project.title">
                        {{ project.title }}
                    </h3>
                    <div v-if="project.__source === 'curseforge'" class="shrink-0 bg-[#f16436] text-white text-[10px] font-bold px-1.5 py-0.5 rounded">
                        CF
                    </div>
                </div>
                
                <p class="text-sm text-content-dimmed line-clamp-2 mt-1 break-words">
                    {{ project.description }}
                </p>
            </div>

            <div class="flex justify-between items-end mt-2">
                <div class="text-xs text-content-dimmed font-mono">
                    <span v-if="project.author">{{ project.author }}</span>
                    <span v-if="project.downloads" class="ml-3 flex items-center inline-flex gap-1">
                        <DownloadIcon class="h-3 w-3" />
                        {{ formatCompactNumber(project.downloads) }}
                    </span>
                </div>

                <button 
                    v-if="instance && project.__source !== 'curseforge'"
                    @click="handleInstall"
                    class="p-2 rounded hover:bg-background-base transition-colors"
                    :class="installed ? 'text-green-500' : 'text-content-base'"
                    :title="installed ? 'Installed' : 'Install'"
                >
                    <component :is="installed ? 'CheckIcon' : 'DownloadIcon'" class="h-5 w-5" />
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
/* Add any specific styles if needed, otherwise Tailwind handles it */
</style>
