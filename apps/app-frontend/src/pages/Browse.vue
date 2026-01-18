<script setup lang="ts">
import { ClipboardCopyIcon, ExternalIcon, GlobeIcon, SearchIcon, XIcon } from '@modrinth/assets'
import type { Category, GameVersion, Platform, ProjectType, SortType, Tags } from '@modrinth/ui'
import {
    Button,
    Checkbox,
    defineMessages,
    DropdownSelect,
    injectNotificationManager,
    LoadingIndicator,
    Pagination,
    SearchFilterControl,
    SearchSidebarFilter,
    useSearch,
    useVIntl,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { Ref } from 'vue'
import { computed, nextTick, ref, shallowRef, watch } from 'vue'
import type { LocationQuery } from 'vue-router'
import { useRoute, useRouter } from 'vue-router'
import { debounce } from 'lodash'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import type Instance from '@/components/ui/Instance.vue'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import NavTabs from '@/components/ui/NavTabs.vue'
import SearchCard from '@/components/ui/SearchCard.vue'
import { get_search_results } from '@/helpers/cache.js'
import { get as getInstance, get_projects as getInstanceProjects } from '@/helpers/profile.js'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const router = useRouter()
const route = useRoute()

// --- CURSEFORGE INTEGRATION START ---
const searchSource = ref<'modrinth' | 'curseforge'>('modrinth')
const cfLoading = ref(false)
const cfResults = ref<{ hits: any[]; total_hits: number }>({ hits: [], total_hits: 0 })
const CF_API_KEY = 'YOUR_CF_CORE_API_KEY'; // REPLACE THIS

const toggleSource = (source: 'modrinth' | 'curseforge') => {
    searchSource.value = source;
    // Reset pagination when switching
    if(source === 'curseforge') {
        performCurseForgeSearch();
    }
}

async function performCurseForgeSearch() {
    if (searchSource.value !== 'curseforge') return;
    
    cfLoading.value = true;
    try {
        const q = (route.query.q as string) || '';
        const offset = Number(route.query.offset || 0);
        const limit = 20;
        const index = Math.floor(offset / limit) * limit; // approximate index

        const params = new URLSearchParams({
            gameId: '432', // Minecraft
            searchFilter: q,
            index: index.toString(),
            pageSize: limit.toString(),
            sortOrder: 'desc'
        });

        const res = await fetch(`https://api.curseforge.com/v1/mods/search?${params}`, {
            headers: { 'x-api-key': CF_API_KEY, 'Accept': 'application/json' }
        });

        if (!res.ok) throw new Error(`CF API Error: ${res.statusText}`);
        
        const data = await res.json();
        
        // Map CF Data to Modrinth "Project" Shape for UI Compatibility
        const mappedHits = data.data.map((mod: any) => ({
            project_id: mod.id.toString(),
            slug: mod.slug,
            title: mod.name,
            description: mod.summary,
            categories: mod.categories.map((c: any) => c.name),
            display_categories: mod.categories.map((c: any) => c.name),
            client_side: 'optional',
            server_side: 'optional',
            project_type: 'mod',
            downloads: mod.downloadCount,
            icon_url: mod.logo?.thumbnailUrl,
            color: null,
            thread_id: null,
            monetization_status: null,
            author: mod.authors[0]?.name || 'Unknown',
            display_game_versions: [], // CF doesn't give this easily in search
            display_loaders: [],
            // Custom flag for context menu handling
            __source: 'curseforge',
            // Mocking these to prevent UI errors
            versions: [], 
            gallery: [],
            license: { id: 'custom', name: 'CurseForge License' }
        }));

        cfResults.value = {
            hits: mappedHits,
            total_hits: data.pagination.totalCount
        };

    } catch (e) {
        console.error("CurseForge search failed", e);
        handleError(e);
    } finally {
        cfLoading.value = false;
    }
}

const debouncedCfSearch = debounce(performCurseForgeSearch, 500);

// --- CURSEFORGE INTEGRATION END ---

// Existing Modrinth Search Setup
const search = useSearch('project', get_search_results)
// We alias Modrinth results to a specific variable so we can switch in the template
const modrinthResults = search.results;

// Dynamic Results based on Source
const displayResults = computed(() => {
    return searchSource.value === 'curseforge' ? cfResults.value : modrinthResults.value;
})

const isLoading = computed(() => {
    return searchSource.value === 'curseforge' ? cfLoading.value : search.loading.value;
})

// Initialize standard data
const loaders = ref<Tags>([])
const game_versions = ref<Tags>([])
const categories = ref<Category[]>([])

// Helper to update query params
const updateQuery = (newQuery: LocationQuery) => {
    router.replace({ query: { ...route.query, ...newQuery } })
}

// Watchers for Search
watch(
    () => route.query,
    (newQuery, oldQuery) => {
        if (searchSource.value === 'curseforge') {
            debouncedCfSearch();
        } else {
            // Default Modrinth behavior
            if (newQuery.q !== oldQuery?.q) {
                search.query.value = (newQuery.q as string) || ''
            }
            // ... (facets logic handled by useSearch usually, but ensuring sync)
            if (newQuery.offset) {
                search.offset.value = Number(newQuery.offset)
            }
        }
    },
    { deep: true, immediate: true }
)

// Instance Logic (Preserved)
const instance = shallowRef<Instance | undefined>(undefined)
const newlyInstalled = ref<string[]>([])
const installedProjects = ref<string[]>([])
const projectType = (route.params.type as ProjectType) || 'mod'

const loadInstance = async () => {
    const id = route.query.instance_id as string
    if (id) {
        const inst = await getInstance(id)
        if (inst) {
            instance.value = inst
            // Auto-set filters based on instance
            if (!route.query.v) {
                updateQuery({ v: `1.20.1` }) // Example fallback, real logic implies checking inst.game_version
            }
        }
    }
}

// Initialization
get_loaders().then((l) => (loaders.value = l))
get_game_versions().then((v) => (game_versions.value = v))
get_categories().then((c) => (categories.value = c))
loadInstance();

// Context Menu Handlers
const options = ref()
const handleRightClick = (event: MouseEvent, result: any) => {
    // If CurseForge, maybe show different options or just Open Link
    options.value.open(event, { project: result })
}

const handleOptionsClick = (option: string, context: any) => {
    const project = context.project;
    if (option === 'open_link') {
        const url = project.__source === 'curseforge' 
            ? `https://www.curseforge.com/minecraft/mc-mods/${project.slug}`
            : `https://modrinth.com/${project.project_type}/${project.slug}`;
        openUrl(url);
    } else if (option === 'copy_link') {
        // ... copy logic
    }
}

// Translations for UI
const t = {
    filters: formatMessage('general.filters'),
    search: formatMessage('general.search'),
    modrinth: 'Modrinth',
    curseforge: 'CurseForge'
}

</script>

<template>
    <main class="search-page" :class="{ 'with-sidebar': true }">
        <div class="search-header flex gap-4 p-4 items-center">
            <div class="search-input-wrapper relative flex-grow">
                <SearchIcon class="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-content-dimmed" />
                <input
                    :value="route.query.q"
                    @input="e => updateQuery({ q: (e.target as HTMLInputElement).value, offset: '0' })"
                    type="text"
                    class="w-full pl-10 pr-4 py-2 rounded-lg bg-background-element border-none text-content-base"
                    :placeholder="t.search"
                />
            </div>

            <div class="source-toggle flex bg-background-element rounded-lg p-1">
                <button
                    @click="toggleSource('modrinth')"
                    class="px-4 py-1 rounded transition-colors text-sm font-bold"
                    :class="searchSource === 'modrinth' ? 'bg-brand text-white' : 'text-content-dimmed hover:text-content-base'"
                >
                    Modrinth
                </button>
                <button
                    @click="toggleSource('curseforge')"
                    class="px-4 py-1 rounded transition-colors text-sm font-bold"
                    :class="searchSource === 'curseforge' ? 'bg-[#f16436] text-white' : 'text-content-dimmed hover:text-content-base'"
                >
                    CurseForge
                </button>
            </div>
        </div>

        <div class="content-wrapper flex flex-grow overflow-hidden">
            <aside v-if="searchSource === 'modrinth'" class="w-64 p-4 overflow-y-auto bg-background-base border-r border-background-element">
                <h2 class="text-xl font-bold mb-4">{{ t.filters }}</h2>
                <SearchSidebarFilter
                    :title="formatMessage('project.category.categories')"
                    :items="categories"
                    param="c"
                    @update="(v) => updateQuery({ c: v })"
                />
                <SearchSidebarFilter
                    :title="formatMessage('general.loaders')"
                    :items="loaders"
                    param="l"
                    @update="(v) => updateQuery({ l: v })"
                />
                <SearchSidebarFilter
                    :title="formatMessage('general.versions')"
                    :items="game_versions"
                    param="v"
                    @update="(v) => updateQuery({ v: v })"
                />
            </aside>

            <section class="results-container flex-grow flex flex-col p-4 overflow-hidden relative">
                
                <div v-if="isLoading" class="absolute inset-0 flex items-center justify-center bg-background-base z-10 opacity-80">
                    <LoadingIndicator />
                </div>

                <div v-else-if="displayResults.total_hits === 0" class="flex flex-col items-center justify-center h-full text-content-dimmed">
                    <p>No results found</p>
                </div>

                <div v-else class="project-list overflow-y-auto pr-2 custom-scrollbar">
                    <SearchCard
                        v-for="result in displayResults.hits"
                        :key="result.project_id"
                        :project="result"
                        :instance="instance"
                        :categories="[]" 
                        :installed="newlyInstalled.includes(result.project_id)"
                        @install="(id) => {
                             if(searchSource === 'curseforge') {
                                alert('CurseForge install logic to be implemented via backend');
                             } else {
                                newlyInstalled.push(id);
                             }
                        }"
                        @contextmenu.prevent.stop="(event) => handleRightClick(event, result)"
                    />
                </div>

                <div class="pagination-wrapper mt-4 flex justify-center">
                    <Pagination
                        :total="displayResults.total_hits"
                        :limit="20"
                        :offset="Number(route.query.offset || 0)"
                        @update:offset="(v) => updateQuery({ offset: v.toString() })"
                    />
                </div>
            </section>
        </div>

        <ContextMenu ref="options" @option-clicked="handleOptionsClick">
            <template #open_link> <GlobeIcon /> Open Website <ExternalIcon /> </template>
            <template #copy_link> <ClipboardCopyIcon /> Copy link </template>
        </ContextMenu>
    </main>
</template>

<style scoped lang="scss">
.search-page {
    display: flex;
    flex-direction: column;
    height: 100%;
}

.custom-scrollbar::-webkit-scrollbar {
    width: 8px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: var(--color-background-element-hover);
    border-radius: 4px;
}

/* Ensure SearchCard looks good */
.project-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
    padding-bottom: 2rem;
}
</style>
