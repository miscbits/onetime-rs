<script setup>
    import { ref } from 'vue';

    const emit = defineEmits({
        // Validate submit event
        getSecretEvent: ({ secret, password }) => {
            if (secretId && password) {
              return true
            } else {
              console.warn('Invalid submit event payload!')
              return false
            }
        }
    })

    function submitForm(secret, password) {
      emit('getSecretEvent', secret, password )
    }
</script>

<template>
    <p>Retrieve a secret</p>
    <input type='text' name='secretId' v-model="secretId" placeholder="secret id" />
    <input v-model="password" type="password" name="password" placeholder="password"/>
    <button @click="submitForm(secretId,password)">submit</button>
</template>
