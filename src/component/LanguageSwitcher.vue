<script setup lang="ts">
    import { ref, computed, nextTick } from 'vue';
    import { useI18n } from 'vue-i18n';

    const { locale } = useI18n();

    const languages = [
        { code: 'ru', flagClass: 'fi-ru', name: 'Русский' },
        { code: 'en', flagClass: 'fi-us', name: 'English' }
    ];

    const isOpen = ref(false);

    const currentLang = computed(() =>
        languages.find(l => l.code === locale.value) || languages[0]
    );

    const otherLanguages = computed(() =>
        languages.filter(l => l.code !== locale.value)
    );

    const toggleMenu = async () => {
        isOpen.value = !isOpen.value;
        if (isOpen.value) {
            await nextTick();
            const close = () => {
                isOpen.value = false;
                window.removeEventListener('click', close);
            };
            window.addEventListener('click', close);
        }
    };

    const setLanguage = (code: string) => {
        locale.value = code;
        isOpen.value = false;
    };
</script>

<template>
    <div class="lang-switcher">
        <div class="lang-node main"
             @click.stop="toggleMenu"
             :class="{ active: isOpen }">
            <span :class="['fi', currentLang.flagClass, 'flag-icon']"></span>
        </div>

        <Transition name="slide-fade">
            <div v-if="isOpen" class="lang-list">
                <div v-for="lang in otherLanguages"
                     :key="lang.code"
                     class="lang-node option"
                     @click="setLanguage(lang.code)">
                    <span :class="['fi', lang.flagClass, 'flag-icon']"></span>
                    <span class="lang-tooltip">{{ lang.name }}</span>
                </div>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
    .lang-switcher {
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: center;
        z-index: 500;
    }

    .lang-node {
        width: 22px;
        height: 22px;
        border-radius: 50%;
        background: #1a1d21;
        border: 1px solid #444c56;
        display: flex;
        justify-content: center;
        align-items: center;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        overflow: hidden; /* Важно для круглых флагов */
    }

    .lang-node:hover {
        border-color: #94a3b8;
        transform: scale(1.1);
    }

    .lang-node.main.active {
        border-color: #00c087;
        box-shadow: 0 0 10px rgba(0, 192, 135, 0.2);
    }


    .flag-icon {
        width: 100%;
        height: 100%;
        background-size: cover;
        background-position: center;
        border-radius: 50%;
        transform: scale(1.5);
    }

    .lang-list {
        position: absolute;
        top: 40px;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .lang-tooltip {
        position: absolute;
        right: 40px;
        background: #2d333b;
        color: white;
        padding: 4px 10px;
        border-radius: 6px;
        font-size: 11px;
        white-space: nowrap;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.2s;
        border: 1px solid #444c56;
        box-shadow: 0 4px 10px rgba(0,0,0,0.3);
    }

    .lang-node.option:hover .lang-tooltip {
        opacity: 1;
    }

    .slide-fade-enter-active, .slide-fade-leave-active {
        transition: all 0.2s ease;
    }

    .slide-fade-enter-from, .slide-fade-leave-to {
        opacity: 0;
        transform: translateY(-10px);
    }
</style>
