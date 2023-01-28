<template>
    <td>
        <div class="cell-wrapper" :class="{ highlight: highlight }" @click="toggleEvents" ref="cellElement">
            <div class="number">{{ day }}</div>
            <div class="dot" v-if="events.length"></div>
        </div>
    </td>
    <Popover :source-rect="rect" v-if="eventsShown">test</Popover>
</template>

<script setup lang="ts">
import type { CalendarEvent } from "@/utils/CalendarEvent";

const props = defineProps<{
    day: number,
    highlight: boolean,
    events: CalendarEvent[]
}>();

const eventsShown = ref(false);
const cellElement = ref<HTMLDivElement | null>(null);

const rect = computed(() => {
    // dummy read to update when body changes
    useBodyRect().value;
    return cellElement.value?.getBoundingClientRect();
});

function toggleEvents() {
    if (props.events.length) {
        eventsShown.value = !eventsShown.value;
    }
}
</script>

<style scoped lang="scss">
.highlight {
    background-color: #AFE9D5;
}

.cell-wrapper {
    border-radius: 4px;
    padding: 8px;
    display: flex;
    flex-flow: row nowrap;
    justify-content: center;
    align-items: center;

    // for position: absolute to work
    position: relative;
}

.number {
    display: inline-block;
}

.dot {
    position: absolute;
    top: 2px;
    right: 2px;
    background-color: tomato;
    border-radius: 50%;
    width: 8px;
    height: 8px;
}
</style>