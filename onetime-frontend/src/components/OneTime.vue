<script setup>
import CreateSecret from './CreateSecret.vue'
import GetSecret from './GetSecret.vue'
import SecretService from '../services/SecretService.js'

const secretService = new SecretService();

async function createSecretCallback(secret, password) {
	let secret_content = await secretService.createSecretRequest(secret, password)
	console.log(secret_content)
}

async function getSecretCallback(secretId, password) {
	let secret_content = await secretService.getSecretRequest(secretId, password)
	console.log('get secret')
}
</script>

<template>
	<CreateSecret @createSecretEvent="createSecretCallback" />

	<GetSecret @getSecretEvent="getSecretCallback" />
</template>