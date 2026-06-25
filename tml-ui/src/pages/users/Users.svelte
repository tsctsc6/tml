<script lang="ts">
  import {
    DataTable,
    DataTableSkeleton,
    Modal,
    Pagination,
    TextInput,
    Toggle,
  } from "carbon-components-svelte";
  import apiClient from "../../lib/api";

  interface ReadAllNormalUserResponse {
    total: number;
    items: UserItem[];
  }

  interface UserItem {
    id: number;
    username: string;
    enabled: boolean;
    created_at: string;
  }

  // Table header
  const headers = [
    { key: "id", value: "Id" },
    { key: "username", value: "Username" },
    { key: "enabled", value: "Enabled" },
    { key: "created_at", value: "Created at" },
  ] as const;

  // Pagination value
  let pageSize = 10;
  let page = 1;

  // Data from backend
  let rows: UserItem[] = [];
  let totalItems: number = 0;
  let loading: boolean = true;

  // Get data from backend
  async function fetchData(currentPage: number, currentPageSize: number) {
    loading = true;
    try {
      const response = await apiClient.get<ReadAllNormalUserResponse>(
        `/mgmt/read_all_normal_user?page_index=${currentPage - 1}&page_size=${currentPageSize}`,
      );
      rows = response.data.items;
      totalItems = response.data.total;
    } catch (error) {
      console.error("Raad all users failed:", error);
    } finally {
      loading = false;
    }
  }

  $: fetchData(page, pageSize);

  interface UpdateNormalUserRequest {
    id: number;
    username: string;
    enabled: boolean;
  }

  interface UpdateNormalUserResponse {}

  // Modal Editing
  let isModalOpen = false;
  let isSubmitting = false;
  // Copy a data, in case user not save
  let editingItem: UpdateNormalUserRequest = {
    id: 0,
    username: "",
    enabled: false,
  };

  // Row click event
  function handleRowClick(event: CustomEvent<{ row: UserItem }>) {
    const clickedRow = event.detail.row;
    editingItem.id = clickedRow.id;
    editingItem.username = clickedRow.username;
    editingItem.enabled = clickedRow.enabled;
    isModalOpen = true;
  }

  // Submit changes to backend
  async function handleSaveChanges() {
    isSubmitting = true;
    try {
      const updateNormalUserRequest: UpdateNormalUserRequest = {
        id: editingItem.id,
        username: editingItem.username,
        enabled: editingItem.enabled,
      };
      const response = await apiClient.post<UpdateNormalUserResponse>(
        "/mgmt/update_normal_user",
        updateNormalUserRequest,
      );
      isModalOpen = false;

      // Update row in datatable
      rows = rows.map((item) => {
        if (item.id === editingItem.id) {
          item.username = editingItem.username;
          item.enabled = editingItem.enabled;
        }
        return item;
      });
    } catch (error) {
      console.error("Update user failed:", error);
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if loading}
  <DataTableSkeleton {headers} rows={pageSize} />
{:else}
  <DataTable title="Users" {headers} {rows} on:click:row={handleRowClick} />
{/if}

<Pagination {totalItems} pageSizes={[5, 10, 20, 50]} bind:pageSize bind:page />

<Modal
  bind:open={isModalOpen}
  modalHeading="Edit"
  primaryButtonText={isSubmitting ? "Saving" : "Save"}
  secondaryButtonText="Cancel"
  primaryButtonDisabled={isSubmitting}
  on:click:button--primary={handleSaveChanges}
  on:click:button--secondary={() => (isModalOpen = false)}
  on:close={() => (isModalOpen = false)}
>
  <div class="edit-form">
    <TextInput
      labelText="Id"
      bind:value={editingItem.id}
      placeholder="Id"
      readonly
    />
  </div>
  <div class="edit-form">
    <TextInput
      labelText="Username"
      bind:value={editingItem.username}
      placeholder="Username"
    />
  </div>
  <div class="edit-form">
    <Toggle labelText="Enabled" bind:toggled={editingItem.enabled} />
  </div>
</Modal>

<style>
  .edit-form :global(.bx--form-item) {
    margin-bottom: 1.25rem;
  }
</style>
