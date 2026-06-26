package svc

// getServiceHealth serves GET /api/v1/services/{id}/health — the source of truth
// for the service health contract.

type getServiceHealthOutput struct {
	Name  string `json:"name"`
	State string `json:"state"`
}

type HealthState string

const (
	HealthStateOperational HealthState = "OPERATIONAL"
	HealthStateDegraded    HealthState = "DEGRADED"
	HealthStateIdle        HealthState = "IDLE"
)

// listServices serves GET /api/v1/services — the source of truth for the service
// list contract. The frontend's ServiceRow matches this exactly.

type serviceRow struct {
	ServiceID   string `json:"service_id"`
	DisplayName string `json:"display_name"`
}
