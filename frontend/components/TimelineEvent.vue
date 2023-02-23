<template>
    <div class="event" :style="{ top, bottom }">
        <div class="event-inner">{{ event.name }}</div>
    </div>
</template>

<script setup lang="ts">
import { CalendarEvent } from "@/utils/CalendarEvent";

const props = defineProps<{
    event: CalendarEvent;
}>();

const top = computed(
    () => `${(props.event.startTime?.diff(props.event.startTime.startOf("day"), "day", true) ?? 0) * 100}%`
);
const bottom = computed(
    () => `${(props.event.endTime?.endOf("day").diff(props.event.endTime, "day", true) ?? 0) * 100}%`
);
</script>

<style scoped>
.event {
    position: absolute;
    padding: 1px;
    width: 100%;
}

.event-inner {
    padding: 0.2em;
    height: 100%;
    border-radius: 0.4em;
    background-color: #f76;
}
</style>
