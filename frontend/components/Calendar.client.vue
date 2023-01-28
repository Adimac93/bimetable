<template>
    <div>This is a calendar</div>
    <table>
        <thead>
            <tr>
                <th colspan="7"><div class="header">
                    <button @click="changeMonth(-1)">&lt;</button>
                    <span>
                        {{ monthStart.format("MMM YYYY") }}
                    </span>
                    <button @click="changeMonth(1)">&gt;</button>
                </div></th>
            </tr>
            <tr>
                <th v-for="weekday in weekdays">{{ weekday }}</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="week in days">
                <template v-for="day in week">
                    <template v-if="'offset' in day">
                        <td v-if="day.offset" :colspan="day.offset"></td>
                    </template>
                    <td v-else>{{ day.date() }}</td>
                </template>
            </tr>
        </tbody>
    </table>
</template>

<script setup lang="ts">
import dayjs from "@/utils/dayjs";

type CalendarSpace = dayjs.Dayjs | {
    offset: number
};

const weekdays = dayjs.weekdaysMin(true);

const monthStart = ref(dayjs("2000-02-03").date(1));
const daysInMonth = computed(() => monthStart.value.daysInMonth());
const days = computed(() => {
    const days: CalendarSpace[][] = [];
    // if it's not the first day of the week, add an initial week
    if (monthStart.value.weekday() != 0) {
        days.push([]);
    }

    for (let i = 0; i < daysInMonth.value; i++) {
        const day = monthStart.value.add(i, "days");
        const weekDay = day.weekday();
        if (weekDay == 0) {
            days.push([]);
        }
        days[days.length - 1].push(day);
    }

    // add empty cells at beginning and end
    const startEmptyCells = monthStart.value.weekday();
    days[0].unshift({ offset: startEmptyCells });

    return days;
});

function changeMonth(offset: number) {
    monthStart.value = monthStart.value.add(offset, "months");
}
</script>

<style scoped>
.header {
    display: flex;
    flex-flow: row nowrap;
    justify-content: space-between;
}
</style>