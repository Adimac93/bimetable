<template>
    <div>
        <form @submit.prevent="submit">
            <div class="row">
                <label for="name">Name</label>
                <input type="text" id="name" v-model="name" />
            </div>
            <div class="row">
                <label for="description">Description</label>
                <textarea id="description" v-model="description"></textarea>
            </div>
            <div class="row">
                <label for="starts-at">Starts at</label>
                <input type="datetime-local" id="starts-at" v-model="startsAt" />
            </div>
            <div class="row">
                <label for="ends-at">Ends at</label>
                <input type="datetime-local" id="ends-at" v-model="endsAt" />
            </div>
            <div class="row">
                <button>Do the thing</button>
            </div>
        </form>
    </div>
</template>

<script setup lang="ts">
const name = ref<string>();
const description = ref<string>();
const startsAt = ref<string>();
const endsAt = ref<string>();

async function submit() {
    const result = await $fetch("/api/events", {
        method: "PUT",
        body: {
            data: {
                payload: {
                    name: name.value,
                    description: description.value,
                },
                startsAt: startsAt.value,
                endsAt: endsAt.value,
            },
        },
    });

    alert(result);
}
</script>
