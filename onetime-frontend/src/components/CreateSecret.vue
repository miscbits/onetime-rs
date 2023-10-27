<script setup>
    import { ref } from 'vue';

    const emit = defineEmits({
        // Validate submit event
        createSecretEvent: ({ secret, password }) => {
            if (secret && password) {
              return true
            } else {
              console.warn('Invalid submit event payload!')
              return false
            }
        }
    })

    function submitForm(secret, password) {
      emit('createSecretEvent', secret, password )
    }
</script>

<template>
    <p>Create a secret</p>
    <textarea v-model="secret" placeholder="secret content"></textarea>
    <input v-model="password" type="password" name="password" placeholder="password"/>
    <button @click="submitForm(secret,password)">submit</button>
</template>
