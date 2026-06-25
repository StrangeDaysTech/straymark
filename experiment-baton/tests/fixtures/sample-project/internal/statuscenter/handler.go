package statuscenter

// getServiceHealth serves GET /api/v1/services/{id}/health — the source of
// truth for the service health contract (per-component since PM-002).

type getServiceHealthOutput struct {
	ServiceID   string              `json:"service_id"`
	State       string              `json:"state"`
	HealthScore int                 `json:"health_score"`
	Components  []componentResponse `json:"components"`
}

type componentResponse struct {
	Name   string `json:"name"`
	State  string `json:"state"`
	Detail string `json:"detail,omitempty"`
}

type HealthState string

const (
	HealthStateOperational HealthState = "OPERATIONAL"
	HealthStateDegraded    HealthState = "DEGRADED"
	HealthStateMajorOutage HealthState = "MAJOR_OUTAGE"
	HealthStateIdle        HealthState = "IDLE"
)
