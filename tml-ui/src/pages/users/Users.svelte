<script lang="ts">
  import {
    Button,
    DataTable,
    DataTableSkeleton,
    Modal,
    Pagination,
    TextInput,
    Toggle,
    Toolbar,
    ToolbarContent,
    PasswordInput,
  } from "carbon-components-svelte";
  import TrashCan from "carbon-icons-svelte/lib/TrashCan.svelte";
  import Add from "carbon-icons-svelte/lib/Add.svelte";
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
    { key: "delete", value: "Delete", empty: true },
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
  let isEditingModalOpen = false;
  let isEditingSubmitting = false;
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
    isEditingModalOpen = true;
  }

  // Submit changes to backend
  async function handleSaveChanges() {
    isEditingSubmitting = true;
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
      isEditingModalOpen = false;

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
      isEditingSubmitting = false;
    }
  }

  interface DeleteNormalUserRequest {
    id: number;
  }

  interface DeleteNormalUserResponse {}

  // Modal Delete
  let isDeleteModalOpen = false;
  let itemToDelete: UserItem | null = null;
  let isDeleting = false;

  function triggerDelete(event: Event, item: UserItem) {
    event.stopPropagation();
    itemToDelete = item;
    isDeleteModalOpen = true;
  }

  async function confirmDelete() {
    if (!itemToDelete) return;
    isDeleting = true;

    try {
      const deleteNormalUserRequest: DeleteNormalUserRequest = {
        id: itemToDelete.id,
      };
      const response = await apiClient.post<DeleteNormalUserResponse>(
        "/mgmt/delete_normal_user",
        deleteNormalUserRequest,
      );
      rows = rows.filter((item) => item.id !== itemToDelete!.id);
      isDeleteModalOpen = false;
    } catch (error) {
      console.error("Delete user failed:", error);
    } finally {
      isDeleting = false;
      itemToDelete = null;
    }
  }

  interface CreateNormalUserRequest {
    username: string;
    password: string;
  }

  interface CreateNormalUserResponse {}

  // Modal Create
  let isCreateModalOpen = false;
  let itemToCreate: CreateNormalUserRequest = {
    username: "",
    password: "",
  };
  let isCreating = false;

  function openAddModal() {
    itemToCreate = {
      username: "",
      password: "",
    };
    isCreateModalOpen = true;
  }

  async function submitCreate() {
    isCreating = true;
    try {
      const response = await apiClient.post<CreateNormalUserResponse>(
        "/mgmt/create_normal_user",
        itemToCreate,
      );
      fetchData(page, pageSize);
    } catch (error) {
      console.error("Create user failed:", error);
    } finally {
      isCreating = false;
      isCreateModalOpen = false;
    }
  }
</script>

{#if loading}
  <DataTableSkeleton {headers} rows={pageSize} />
{:else}
  <DataTable title="Users" {headers} {rows} on:click:row={handleRowClick}>
    <Toolbar>
      <ToolbarContent>
        <Button icon={Add} iconDescription="Add" on:click={openAddModal}
        ></Button>
      </ToolbarContent>
    </Toolbar>
    <svelte:fragment slot="cell" let:cell let:row>
      {#if cell.key === "delete"}
        <Button
          kind="danger-tertiary"
          size="small"
          iconDescription="Delete"
          icon={TrashCan}
          on:click={(e) => triggerDelete(e, row as UserItem)}
        />
      {:else}
        {cell.value}
      {/if}
    </svelte:fragment>
  </DataTable>
{/if}

<Pagination {totalItems} pageSizes={[5, 10, 20, 50]} bind:pageSize bind:page />

<Modal
  bind:open={isEditingModalOpen}
  modalHeading="Edit"
  primaryButtonText={isEditingSubmitting ? "Saving" : "Save"}
  secondaryButtonText="Cancel"
  primaryButtonDisabled={isEditingSubmitting}
  on:click:button--primary={handleSaveChanges}
  on:click:button--secondary={() => (isEditingModalOpen = false)}
  on:close={() => (isEditingModalOpen = false)}
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

<Modal
  danger
  bind:open={isDeleteModalOpen}
  modalHeading="Are you sure to delete?"
  primaryButtonText={isDeleting ? "Deleting..." : "Delete"}
  secondaryButtonText="Cancael"
  primaryButtonDisabled={isDeleting}
  on:click:button--primary={confirmDelete}
  on:click:button--secondary={() => (isDeleteModalOpen = false)}
  on:close={() => (isDeleteModalOpen = false)}
>
  <p>
    Are you sure to delete user <strong>{itemToDelete?.id}</strong> ?
  </p>
  <p>All relevant information will be lost.</p>
</Modal>

<Modal
  bind:open={isCreateModalOpen}
  modalHeading="Edit"
  primaryButtonText={isCreating ? "Creating" : "Create"}
  secondaryButtonText="Cancel"
  primaryButtonDisabled={isCreating}
  on:click:button--primary={submitCreate}
  on:click:button--secondary={() => (isCreateModalOpen = false)}
  on:close={() => (isCreateModalOpen = false)}
>
  <div class="edit-form">
    <TextInput
      labelText="Username"
      bind:value={itemToCreate.username}
      placeholder="Username"
    />
  </div>
  <div class="edit-form">
    <PasswordInput
      labelText="Passowrd"
      bind:value={itemToCreate.password}
      placeholder="Passowrd"
    />
  </div>
</Modal>

<style>
  .edit-form :global(.bx--form-item) {
    margin-bottom: 1.25rem;
  }
</style>
