<script lang="ts">
  import {
    Button,
    Column,
    Form,
    Grid,
    PasswordInput,
    Row,
    TextInput,
  } from "carbon-components-svelte";
  import apiClient from "../../lib/api";

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
    const loginRequest: LoginRequest = { username, password };
    const response = await apiClient.post<LoginResponse>(
      "/auth/login",
      loginRequest,
    );
    console.log("Login result:", response);
    if (typeof window === "undefined") {
      return;
    }
    localStorage.setItem("token", response.data.token);
    window.location.reload();
  }
</script>

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
