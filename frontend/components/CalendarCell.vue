<template>
    <td>
        <div class="cell-wrapper" :class="{ highlight: highlight }" @click="toggleEvents" ref="cellElement">
            <div class="number">{{ day }}</div>
            <div class="dot" v-if="events.length"></div>
        </div>
    </td>
</template>

<script setup lang="ts">
import type { CalendarEvent } from "@/utils/CalendarEvent";

const props = defineProps<{
    day: number,
    highlight: boolean,
    events: CalendarEvent[]
}>();

const cellElement = ref<HTMLDivElement | null>(null);

const emit = defineEmits<{
    (event: "activate"): void
}>();

function toggleEvents() {
    emit("activate");
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