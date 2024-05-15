<script lang="ts">
	import { goto } from '$app/navigation';
	import { LoginStore } from '$houdini';

	const login = new LoginStore();

	let email = '';
	let password = '';

	function auth() {
		login
			.mutate({
				email,
				password
			})
			.then((user) => {
				if (user.data != null && user.data != undefined) {
					console.log(user.data);
					localStorage.setItem('user', JSON.stringify(user.data.users.login));
					goto("/");
				}
			});
	}
</script>

<div class="center-container">
	<div>
		<form on:submit|preventDefault={auth}>
			<label>
				Email
				<input bind:value={email} placeholder="Enter your email" />
			</label>
			<label>
				Password
				<input type="password" bind:value={password} placeholder="Enter your password" />
			</label>
			<button on:click={auth}>Submit</button>
		</form>
	</div>
</div>

<style>
	.center-container {
		display: flex;
		justify-content: center;
		align-items: center;
		height: 100vh;
		background-color: #f0f0f0;
		flex-direction: column;
	}

	form {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 20px;
		border-radius: 5px;
		background-color: white;
		box-shadow: 0px 0px 10px rgba(0, 0, 0, 0.1);
	}

	label {
		padding: 10px;
		border-radius: 5px;
		border: 1px solid #ccc;
		display: flex;
		justify-content: space-between;
	}

	input {
		margin-left: 10px;
	}
</style>
