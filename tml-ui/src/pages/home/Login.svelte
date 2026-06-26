<script lang="ts">
  import {
    Button,
    Column,
    Form,
    Grid,
    NotificationQueue,
    PasswordInput,
    Row,
    TextInput,
  } from "carbon-components-svelte";
  import { apiClientExt } from "../../lib/api";

  let queue: NotificationQueue;

  interface Props {
    onLoginSuccess: () => void;
  }
  let { onLoginSuccess }: Props = $props();

  interface LoginRequest {
    username: string;
    password: string;
  }

  interface LoginResponse {
    token: string;
  }

  let username: string = "";
  let password: string = "";

  async function handleFormSubmit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    try {
      const loginRequest: LoginRequest = { username, password };
      const response = await apiClientExt.post<LoginResponse>(
        "/auth/login",
        loginRequest,
      );
      if (typeof window === "undefined") {
        return;
      }
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }
      localStorage.setItem("token", response.data.token);
      onLoginSuccess();
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    }
  }
</script>

<NotificationQueue bind:this={queue} />

<Grid>
  <Row>
    <Column><h2>Login</h2></Column>
  </Row>
  <Row>
    <Column>
      <Form on:submit={handleFormSubmit}>
        <TextInput
          labelText="Username"
          placeholder="Enter username..."
          bind:value={username}
        />
        <PasswordInput
          labelText="Password"
          placeholder="Enter password..."
          bind:value={password}
        />
        <Button type="submit">Submit</Button>
      </Form>
    </Column>
  </Row>
</Grid>
